use actix_web::{ web::{ Data, Json, Query }, HttpResponse, Responder };
use mongodb::bson::doc;
use serde::{ Deserialize, Serialize };
use std::{ collections::HashMap, sync::{ Arc, Mutex } };

use crate::{db::{ User, DB }, settings::Settings};
use crate::ledger::{ Ledger, Tx };

pub struct AppData {
    pub db: DB,
    pub ledger: Ledger,
    pub settings: Settings
}

#[derive(Deserialize)]
pub struct NewTxRequestBody {
    from_addr: String,
    to_addr: String,
    amt: u32,
    secret: String,
}

#[derive(Deserialize)]
pub struct AddrLookupRequestBody {
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Serialize)]
struct RequestFailure<'a> {
    status: &'a str,
    justification: &'a str,
}

#[derive(Serialize)]
struct TxLedgerResponseBody {
    ledger: HashMap<String, u32>,
}

#[derive(Serialize)]
struct Users {
    users: Vec<User>,
}

fn reject(justification: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(RequestFailure {
        status: "fail",
        justification,
    })
}

pub async fn handle_tx(
    req: Json<NewTxRequestBody>,
    data: Data<Arc<Mutex<AppData>>>
) -> impl Responder {
    let AppData { db, ledger } = &mut *data.lock().expect("failed to lock app data mutex");
    let NewTxRequestBody { amt, from_addr, to_addr, secret } = req.into_inner();

    if db.fetch_by_addr(&to_addr).await.is_none() {
        return reject("no recieving user could be found with the provided address");
    }

    match (db.fetch_by_addr(&from_addr).await, db.fetch_by_addr(&to_addr).await) {
        (None, _) => reject("could not find sender with address provided"),
        (_, None) => reject("could not find recipient with address provided"),
        (Some(User { bryxcoin_password, .. }), _) if bryxcoin_password != secret =>
            reject("invalid secret"),
        (Some(User { bryxcoin_address, .. }), _) =>
            match ledger.balances.get(&bryxcoin_address) {
                None => reject("specified sender address has no balance"),
                Some(bal) if *bal < amt => reject("specified sender address has insuffient funds"),
                Some(_) => {
                    let tx = Tx {
                        amt,
                        from_addr,
                        to_addr,
                    };

                    ledger.new_tx(&tx);
                    HttpResponse::Created().json(&tx)
                }
            }
    }
}

pub async fn handle_users(
    req: Query<AddrLookupRequestBody>,
    data: Data<Arc<Mutex<AppData>>>
) -> impl Responder {
    let AppData { db, .. } = &*data.lock().expect("failed to lock app data mutex");

    let filter = match (&req.first_name, &req.last_name) {
        (Some(first_name), Some(last_name)) =>
            Some(
                doc! {
            "first_name": first_name,
            "last_name": last_name
        }
            ),
        (Some(first_name), None) => Some(doc! { "first_name": first_name }),
        (None, Some(last_name)) => Some(doc! { "last_name": last_name }),
        (None, None) => None,
    };

    if filter == None {
        return reject("specify at least one of the following: 'first_name', 'last_name'");
    }

    HttpResponse::Ok().json(Users {
        users: db
            .fetch_users(filter).await
            .expect("failed to query against the users collection")
            .into_iter()
            .map(|mut u| {
                u.bryxcoin_password = "<<REDACTED>>".to_owned();
                return u;
            })
            .collect(),
    })
}

pub async fn get_txs(data: Data<Arc<Mutex<AppData>>>) -> impl Responder {
    let AppData { ledger, .. } = &mut *data.lock().expect("failed to lock!");
    HttpResponse::Ok().json(TxLedgerResponseBody {
        ledger: ledger.balances.clone(),
    })
}