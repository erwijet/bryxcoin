use std::sync::{ Arc, Mutex };
use ledger::Ledger;
use db::DB;
use ledger::get_ledger_repo_path;
use actix_web::{ HttpResponse, HttpServer, App, web::{ self, Data } };

mod db;
mod ledger;
mod http;

const REMOTE: &str = "git@github.com:bryxcoin/ledger.git";
const BANK_ADDR: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = DB::init().await.expect("failed to establish a connection with mongodb");

    let mut ledger = Ledger::init();
    ledger.compute_balances();


    for (k, v) in &ledger.balances {
        println!("{}: {} bxcn", k, v);
    }

    let data = Arc::new(
        Mutex::new(http::AppData {
            ledger,
            db,
        })
    );

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(data.clone()))
            .route(
                "/health",
                web::get().to(|| HttpResponse::Ok().body("ok"))
            )
            .route("/tx", web::post().to(http::handle_tx))
            .route("/addr", web::get().to(http::handle_addr))
            .route("/balances", web::get().to(http::get_txs) )
    })
        .bind(("0.0.0.0", 8080))?
        .run().await
        .and_then(|_| {
            std::fs::remove_dir_all(get_ledger_repo_path()).expect("failed to cleanup pwd/ledger!");
            println!("[Done]");

            Ok(())
        })
}