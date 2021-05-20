use mongodb::{
    doc,
    coll::options::IndexOptions,
    Bson,
    Client,
    ThreadedClient,
    CommandType,
    db::{Database, ThreadedDatabase}
};
use bson::ordered::OrderedDocument;
use crate::constants::{MONGO_DB, MONGO_COLL_DOCUMENTS};
use crate::errors::*;
use serde_json::json;

mod public_db;

#[derive(Clone)]
pub struct DataStore {
    client: Client,
    database: Database
}

pub trait DataStoreApi{
    fn clean_db(&self) -> Result<bool>;
    fn create_indexes(&self) -> Result<bool>;
    fn db_exists(&self) -> Result<bool>;
    fn metrics(&self) -> Result<serde_json::Value>;
    fn new(host: &str, port: u16) -> Self;
    fn statistics(&self) -> Result<serde_json::Value>;
}

impl DataStoreApi for DataStore {

    fn clean_db(&self) -> Result<bool> {
        info!("cleaning mongoDBs:");
        // create the collections
        let mut success = true;
        success = success & drop_table(&self.database, MONGO_COLL_DOCUMENTS)?;
        return Ok(success);
    }

    //TODO: check the indexes
    fn create_indexes(&self) -> Result<bool> {
        info!("creating indexes");
        let mut success = true;
        let id_index = doc! {
            "id": "text"
        };
        let mut index_unique = IndexOptions::new();
        index_unique.unique = Some(true);
        success = success & create_table(&self.database, MONGO_COLL_DOCUMENTS, id_index.clone(), Some(index_unique.clone()))?;
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

    fn new(host: &str, port: u16) -> DataStore{
        let client = Client::connect(host, port)
            .expect("Failed to initialize mongodb client.");
        DataStore {
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

pub fn create_table(db: &Database, table: &str, key: OrderedDocument, options: Option<IndexOptions>) -> Result<bool>{
    // create index for cts
    let coll = db.collection(table);
    let name = coll.create_index(key, options)?;
    debug!("Created table {} with index {}", table, name);
    Ok(true)
}

pub fn drop_table(db: &Database, table: &str) -> Result<bool>{
    db.drop_collection(table)?;
    debug!("Dropped table {}", table);
    Ok(true)
}