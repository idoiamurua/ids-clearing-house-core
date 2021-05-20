// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// These tests all access the db, so if you run the tests use
// cargo test -- --test-threads=1
// otherwise they will interfere with each other
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
const TEST_CONFIG_FILE: &'static str = "./config.yml";

use core_lib::errors::*;
use core_lib::util;
use crate::db::{DataStoreApi, KeyStore};
use crate::model::doc_type::DocumentType;

fn db_setup() -> KeyStore {
    let config = util::load_config(TEST_CONFIG_FILE);

    let db: KeyStore = util::configure_db(&config).unwrap();
    if let Err(e) = db.clean_db(){
        panic!("Error while cleaning up database {:?}", e);
    }
    if let Err(e) = db.create_indexes(){
        panic!("Error while setting up database {:?}", e);
    };
    db
}

fn tear_down(db: KeyStore){
    if let Err(e) = db.clean_db(){
        panic!("Error while tearing down database {:?}", e);
    }
}

/// Testcase: Document type exists
#[test]
fn test_document_type_exists() -> Result<()>{
    // empty db and create tables
    let db = db_setup();

    // prepare test data
    let dt = DocumentType::new(String::from("test_document_type_exists_dt_dt"), String::from("test_document_type_exists_dt_pid"), vec!());
    db.add_document_type(dt.clone())?;

    // run the test: db should find document type
    assert_eq!(db.exists_document_type(&dt.pid, &dt.id)?, true);

    // clean up
    tear_down(db);

    Ok(())
}


/// Testcase: Document type exists for other pid and is not found
#[test]
fn test_document_type_exists_for_other_pid() -> Result<()>{
    // empty db and create tables
    let db = db_setup();

    // prepare test data
    let dt = DocumentType::new(String::from("test_document_type_exists_for_other_pid_dt"), String::from("test_document_type_exists_for_other_pid_pid"), vec!());
    let wrong_pid = String::from("the_wrong_pid");
    db.add_document_type(dt.clone())?;

    // run the test: db should not find the document type
    assert_eq!(db.exists_document_type(&wrong_pid, &dt.id)?, false);

    // clean up
    tear_down(db);

    Ok(())
}

/// Testcase: Delete on document type with correct pid results in deletion of document type
#[test]
fn test_delete_document_type_correct_pid() -> Result<()>{
    // empty db and create tables
    let db = db_setup();

    // prepare test data and insert into db
    let dt = DocumentType::new(String::from("test_delete_document_type_correct_pid_id"), String::from("test_delete_document_type_correct_pid_pid"), vec!());
    let dt2 = DocumentType::new(String::from("test_delete_document_type_correct_pid_id"), String::from("test_delete_document_type_correct_pid_pid_2"), vec!());
    db.add_document_type(dt.clone())?;
    db.add_document_type(dt2.clone())?;

    // run the test
    db.delete_document_type(&dt.id, &dt.pid)?;

    // db should not find document type
    assert_eq!(db.exists_document_type(&dt.pid, &dt.id)?, false);

    // clean up
    tear_down(db);

    Ok(())
}

/// Testcase: Delete on document type with wrong pid results not in the deletion of document type
#[test]
fn test_delete_document_type_wrong_pid() -> Result<()>{
    // empty db and create tables
    let db = db_setup();

    // prepare test data and insert into db
    let dt = DocumentType::new(String::from("test_delete_document_type_correct_pid_id"), String::from("test_delete_document_type_correct_pid_pid"), vec!());
    let wrong_pid = String::from("the_wrong_pid");
    db.add_document_type(dt.clone())?;

    // run the test
    db.delete_document_type(&dt.id, &wrong_pid)?;

    // db should still find document type
    assert_eq!(db.exists_document_type(&dt.pid, &dt.id)?, true);

    // clean up
    tear_down(db);

    Ok(())
}