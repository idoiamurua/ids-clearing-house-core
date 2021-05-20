use mongodb::{
    doc,
    bson,
    coll::options::{
        FindOptions,
        FindOneAndUpdateOptions
    },
    Bson,
    db::ThreadedDatabase
};
use core_lib::constants::{MONGO_DT_ID, MONGO_ID, MONGO_PID, MONGO_COLL_DOC_TYPES};
use core_lib::errors::*;
use crate::db::KeyStore;
use crate::model::doc_type::DocumentType;

impl KeyStore {
    // DOCTYPE
    pub fn add_document_type(&self, doc_type: DocumentType) -> Result<()> {
        // The model type collection
        let coll = self.database.collection(MONGO_COLL_DOC_TYPES);
        match mongodb::to_bson(&doc_type) {
            Ok(serialized) => {
                match serialized.as_document() {
                    Some(dt) => {
                        match coll.insert_one(dt.clone(), None) {
                            Ok(res) => {
                                debug!("inserted doc_type: acknowledged:{:?} inserted_id:{:?}", res.acknowledged, res.inserted_id);
                                Ok(())
                            },
                            Err(e) => bail!("MongoDB insert_one failed: {}", e)
                        }
                    },
                    _ => bail!("conversion to model type failed!"),
                }
            },
            _ => bail!("conversion to bson failed!")
        }
    }

    //TODO: Do we need to check that no documents of this type exist before we remove it from the db?
    pub fn delete_document_type(&self, id: &String, pid: &String) -> Result<bool> {
        // The model type collection
        let coll = self.database.collection(MONGO_COLL_DOC_TYPES);
        let result = coll.delete_one(doc! { MONGO_ID => id, MONGO_PID => pid }, None)?;
        if result.deleted_count >= 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// checks if the model exits
    pub fn exists_document_type(&self, pid: &String, dt_id: &String) -> Result<bool> {
        // The model collection
        let coll = self.database.collection(MONGO_COLL_DOC_TYPES);
        let result = coll.find_one(Some(doc! { MONGO_ID => dt_id, MONGO_PID => pid }), None)?;
        match result {
            Some(_r) => Ok(true),
            None => {
                debug!("document type with id {} and pid {:?} does not exist!", &dt_id, &pid);
                Ok(false)
            }
        }
    }

    pub fn get_document_types(&self) -> Result<Vec<DocumentType>> {
        // The model type collection
        let mut ret = vec![];
        let coll = self.database.collection(MONGO_COLL_DOC_TYPES);
        let mut options = FindOptions::new();
        options.sort = Some(doc! { MONGO_DT_ID: 1 });
        return match coll.find(None, Some(options)) {
            Ok(mut cursor) => {
                loop {
                    if cursor.has_next()? {
                        // we checked has_next() so unwrap() is safe to get to the Result
                        let u = cursor.next().unwrap()?;
                        ret.push(mongodb::from_bson::<DocumentType>(Bson::Document(u))?);
                    } else {
                        break;
                    }
                }
                Ok(ret)
            }
            Err(e) => Err(format!("no document types found: {}", e.to_string()).into())
        }
    }

    pub fn get_document_type(&self, dt_id: &String) -> Result<DocumentType> {
        // The model type collection
        let coll = self.database.collection(MONGO_COLL_DOC_TYPES);
        let query = Some(doc! { MONGO_ID => dt_id });
        debug!("get_document_type query: {:?}", query);
        return match coll.find_one(query, None)? {
            Some(r) => {
                debug!("doc type found");
                Ok(mongodb::from_bson::<DocumentType>(Bson::Document(r))?)
            },
            None => {
                debug!("doc type not found !");
                Err(format!("document type {} was not found", dt_id).into())
            }
        }
    }

    pub fn update_document_type(&self, doc_type: DocumentType, id: &String) -> Result<bool> {
        // The model type collection
        let coll = self.database.collection(MONGO_COLL_DOC_TYPES);
        let serialized_doc = mongodb::to_bson(&doc_type).unwrap(); // Serialize

        let mut options = FindOneAndUpdateOptions::new();
        options.upsert = Some(true);
        let query = doc! { MONGO_ID => id, MONGO_PID => &doc_type.pid };
        match coll.find_one_and_replace(
            query,
            serialized_doc.as_document().unwrap().clone(),
            Some(options)
        ) {
            Ok(r) => {
                let old_type = mongodb::from_bson::<DocumentType>(Bson::Document(r.unwrap()));
                //TODO might panic
                debug!("old model type was: {}", old_type.ok().unwrap().id);
                Ok(true)
            },
            Err(e) => {
                warn!("model type with id {} and pid {} could not be updated: {:?}", &doc_type.id, &doc_type.pid, e.to_string());
                Ok(false)
            }
        }
    }
}