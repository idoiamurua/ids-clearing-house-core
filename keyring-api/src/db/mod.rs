use mongodb::{
    doc,
    bson,
    coll::options::IndexOptions,
    Bson,
    Client,
    ThreadedClient,
    CommandType,
    db::{
        Database,
        ThreadedDatabase
    }
};

use core_lib::constants::{MONGO_DB, MONGO_COLL_DOC_TYPES, MONGO_COLL_MASTER_KEY, MONGO_MKEY};
use core_lib::db::{DataStoreApi, drop_table, create_table};
use core_lib::errors::*;
use serde_json::json;
use crate::db::crypto::MasterKey;

pub mod crypto;
pub mod doc_type;
#[cfg(test)] mod tests;

#[derive(Clone)]
pub struct KeyStore {
    client: Client,
    database: Database
}

impl DataStoreApi for KeyStore {

    fn clean_db(&self) -> Result<bool> {
        info!("cleaning mongoDBs:");
        // create the collections
        let mut success = true;
        success = success & drop_table(&self.database, MONGO_COLL_DOC_TYPES)?;
        success = success & drop_table(&self.database, MONGO_COLL_MASTER_KEY)?;
        return Ok(success);
    }

    //TODO: check the indexes
    fn create_indexes(&self) -> Result<bool> {
        info!("creating indexes");
        let mut success = true;
        let name_index = doc! {
            "name": 1
        };
        let mut index_unique = IndexOptions::new();
        index_unique.unique = Some(true);
        success = success & create_table(&self.database, MONGO_COLL_DOC_TYPES, name_index.clone(), None)?;
        success = success & create_table(&self.database, MONGO_COLL_MASTER_KEY, name_index.clone(), None)?;
        return Ok(success);
    }

    fn db_exists(&self) -> Result<bool> {
        Ok(self.database.list_collections(None)?.count() > 0)
    }

    fn metrics(&self) -> Result<serde_json::Value> {
        let cmd = doc! { "serverStatus": 1, "repl": 0, "metrics": 1, "locks": 0 };
        let result = self.database.command(cmd, CommandType::Suppressed, None)?;
        Ok(json!(Bson::Document(result)))
    }

    fn new(host: &str, port: u16) -> KeyStore{
        let client = Client::connect(host, port)
            .expect("Failed to initialize mongodb client.");
        KeyStore {
            client: client.clone(),
            database: client.db(MONGO_DB)
        }
    }

    fn statistics(&self) -> Result<serde_json::Value> {
        let cmd = doc! { "dbStats": 1, "scale": 1024 };
        let result = self.database.command(cmd, CommandType::Suppressed, None)?;
        Ok(json!(Bson::Document(result)))
    }
}

impl KeyStore {
    /// we assume there's only one key in the table
    pub fn store_master_key(&self, key: MasterKey) -> Result<bool> {
        let coll = self.database.collection(MONGO_COLL_MASTER_KEY);
        match coll.find_one(None, None)? {
            Some(_r) => {
                bail!("master key already exists")
            },
            None => {
                match coll.insert_one(doc!{MONGO_MKEY: serde_json::to_string(&key)?}, None) {
                    Ok(_res) => {
                        Ok(true)
                    },
                    Err(e) => {
                        bail!("master key could not be stored: {}",e)
                    }
                }
            }
        }
    }

    /// we assume there's only one key in the table
    pub fn get_msk(&self) -> Result<MasterKey> {
        let coll = self.database.collection(MONGO_COLL_MASTER_KEY);
        // there's only one
        let result = coll.find_one(None, None)?;
        match result{
            Some(document) => {
                if let Some(msk) = document.get(MONGO_MKEY).and_then(Bson::as_str) {
                    debug!("msk: {}", msk);
                    Ok(serde_json::from_str::<MasterKey>(&msk).unwrap())
                }  else {
                    Err("no msk found".to_string().into())
                }
            },
            None => bail!("msk not found!"),
        }
    }

}