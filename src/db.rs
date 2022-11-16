use std::str;

use futures::StreamExt;
use mongodb::bson::Bson;
use mongodb::bson::document::ValueAccessError;
use mongodb::bson::{ doc, document::Document };
use mongodb::{ options::ClientOptions, Client, Collection };
use serde::{ Serialize, Deserialize };
use serde_json::Value;

type MongoResult<T> = std::result::Result<T, mongodb::error::Error>;

const DB_NAME: &str = "bagelbot";
const COLL: &str = "users";

const FIRST_NAME: &str = "first_name";
const LAST_NAME: &str = "last_name";

const BRYXCOIN_ADDRESS: &str = "bryxcoin_address";
const BRYXCOIN_PASSWORD: &str = "bryxcoin_password";

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub bryxcoin_address: String,
    pub bryxcoin_password: String,
}

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
}

impl DB {
    pub async fn init() -> MongoResult<Self> {
        let conn_str = std::env::var("MONGO_CONN_STR").expect("$MONGO_CONN_STR is not set!");
        let mut client_options = ClientOptions::parse(&conn_str).await?;

        client_options.app_name = Some("bryxcoin".to_string());

        Ok(Self {
            client: Client::with_options(client_options)?,
        })
    }

    pub async fn fetch_by_addr(&self, addr: &str) -> Option<User> {
        self.get_collection()
            .find_one(doc! { "bryxcoin_address": addr }, None).await
            .expect("failed to query users collection")
            .and_then(|doc| { doc.try_into().ok() })
    }

    pub async fn fetch_users(
        &self,
        filter: impl Into<Option<Document>>
    ) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        let mut cursor = self.get_collection().find(filter, None).await?;
        let mut res: Vec<User> = Vec::new();

        while let Some(Ok(doc)) = cursor.next().await {
            match doc.try_into() {
                Ok(user) => res.push(user),
                Err(err) => {
                    return Err(Box::new(err));
                }
            }
        }

        Ok(res)
    }

    fn get_collection(&self) -> Collection {
        self.client.database(DB_NAME).collection(COLL)
    }
}

impl Into<User> for Document {
    fn into(self) -> User {
        let mut vals = [
                "first_name",
                "last_name",
                "bryxcoin_address",
                "bryxcoin_password",
            ];
        vals = vals.
                .into_iter()
                .map(|s| self.get_str(s))
                .collect::<Result<Vec<&str>, ValueAccessError>>();
        {
            User { first_name: **first_name, last_name: **last_name, bryxcoin_address: **bryxcoin_address, bryxcoin_password: **bryxcoin_password }
        } else {
            panic!("failed to convert Document into User");
        }
    }
}