use std::str;

use futures::StreamExt;
use mongodb::bson::{ doc, document::Document };
use mongodb::{ options::ClientOptions, Client, Collection };
use serde::{ Serialize, Deserialize };

type MongoResult<T> = std::result::Result<T, mongodb::error::Error>;

const DB_NAME: &str = "bagelbot";
const COLL: &str = "users";

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
        User {
            first_name: self.get_str("first_name").expect("failed to access value 'first_name'").to_owned(),
            last_name: self.get_str("last_name").expect("failed to access value 'last_name'").to_owned(),
            bryxcoin_address: self.get_str("bryxcoin_address").expect("failed to access value 'bryxcoin_address'").to_owned(),
            bryxcoin_password: self.get_str("bryxcoin_password").expect("failed to access value 'bryxcoin_password'").to_owned()
        }
    }
}