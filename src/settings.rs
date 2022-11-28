use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub port: u16,
    pub public_key: String,
    pub private_key: String,
    pub ledger_repo: String,
    pub mongo_connection_string: String,
    pub mongo_user_database: String,
    pub mongo_user_collection: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = Config::builder();

        // read the local file only in development mode
        #[cfg(debug_assertions)]
        let builder = builder.add_source(File::with_name("assets/bryxcoin.ini"));

        #[cfg(not(debug_assertions))]
        let builder = builder.add_source(File::with_name("/usr/local/etc/bryxcoin.ini"));

        let config = builder.build()?;
        config.try_deserialize()
    }
}
