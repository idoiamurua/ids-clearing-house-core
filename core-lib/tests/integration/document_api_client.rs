// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// These tests are integration tests and need an up-and-running keyring-api and
// document-api. Use config.yml to configure the urls correctly.
// Before running the tests make sure that there's a valid token in auth/mod.rs
// Also note: Clean up will not work if a test fails.
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
use core_lib::constants::{CONFIG_FILE, DOCUMENT_API_URL};
use core_lib::util;
use core_lib::errors::*;
use core_lib::api::client::document_api::DocumentApiClient;
use crate::{TOKEN, create_test_document, delete_test_doc_type_from_keyring, insert_test_doc_type_into_keyring};

/// Testcase: Standard case: retrieve document
#[test]
fn test_get_document() -> Result<()>{
    // configure client_api
    let config = util::load_config(CONFIG_FILE);
    let doc_api: DocumentApiClient = util::configure_api(DOCUMENT_API_URL, &config)?;

    // prepare test data
    let dt_id = String::from("test_get_document_type_1");
    let pid = String::from("test_get_document_process_1");
    let expected_doc = create_test_document(&pid, &dt_id);
    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    // create test data in db
    doc_api.create_document(&TOKEN.to_string(), &expected_doc)?;

    // run test
    let result = doc_api.get_document(&TOKEN.to_string(), &pid, &expected_doc.id)?;
    println!("Result: {:?}", result);

    // checks
    // ids should match
    assert_eq!(result.id, expected_doc.id);

    // same document type
    assert_eq!(result.dt_id, expected_doc.dt_id);

    // checking the parts
    for i in 0..result.parts.len()-1{
        assert_eq!(expected_doc.parts[i].name, result.parts[i].name);
        assert_eq!(expected_doc.parts[i].content, result.parts[i].content);
    }

    // clean up
    assert!(doc_api.delete_document(&TOKEN.to_string(), &expected_doc.pid, &expected_doc.id)?);

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    Ok(())
}

/// Testcase: Standard case: retrieve document with integrity check
#[test]
fn test_get_document_with_hash() -> Result<()>{
    // configure client_api
    let config = util::load_config(CONFIG_FILE);
    let doc_api: DocumentApiClient = util::configure_api(DOCUMENT_API_URL, &config)?;

    // prepare test data
    let dt_id = String::from("test_get_document_with_hash_type_1");
    let pid = String::from("test_get_document_with_hash_process_1");
    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    let expected_doc = create_test_document(&pid, &dt_id);
    // create test data in db
    let hash_message = doc_api.create_document(&TOKEN.to_string(), &expected_doc)?;
    let expected_hash = hash_message.hash;

    // run test
    let result = doc_api.get_document_with_integrity_check(&TOKEN.to_string(), &pid, &expected_doc.id, &expected_hash)?;
    println!("Result: {:?}", result);

    // checks
    // ids should match
    assert_eq!(result.id, expected_doc.id);

    // same document type
    assert_eq!(result.dt_id, expected_doc.dt_id);

    // checking the parts
    for i in 0..result.parts.len()-1{
        assert_eq!(expected_doc.parts[i].name, result.parts[i].name);
        assert_eq!(expected_doc.parts[i].content, result.parts[i].content);
    }

    // clean up
    assert!(doc_api.delete_document(&TOKEN.to_string(), &expected_doc.pid, &expected_doc.id)?);

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    Ok(())
}

/// Testcase: Retrieve all documents for pid, but there are no documents
#[test]
fn test_get_no_documents_for_pid() -> Result<()>{
    // configure client_api
    let config = util::load_config(CONFIG_FILE);
    let doc_api: DocumentApiClient = util::configure_api(DOCUMENT_API_URL, &config)?;

    // prepare test data
    let dt_id = String::from("test_get_no_documents_for_pid_type");
    let pid_with_doc = String::from("test_get_no_documents_for_pid_pid_1");
    let pid_without_doc = String::from("test_get_no_documents_for_pid_pid_2");
    let expected_doc = create_test_document(&pid_with_doc, &dt_id);
    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid_with_doc, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid_with_doc, &dt_id)?;

    // create test data in db
    doc_api.create_document(&TOKEN.to_string(), &expected_doc)?;

    // run test
    let result = doc_api.get_documents_for_pid(&TOKEN.to_string(), &pid_without_doc)?;
    println!("Result: {:?}", result);

    // check that there are no documents found
    assert_eq!(result.len(), 0);

    // clean up
    assert!(doc_api.delete_document(&TOKEN.to_string(), &expected_doc.pid, &expected_doc.id)?);

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid_with_doc, &dt_id)?;

    Ok(())
}

/// Testcase: Standard case: Retrieve all documents for pid
//TODO
#[test]
fn test_get_documents_for_pid() -> Result<()>{
    // configure client_api
    let config = util::load_config(CONFIG_FILE);
    let doc_api: DocumentApiClient = util::configure_api(DOCUMENT_API_URL, &config)?;

    // prepare test data
    let dt_id = String::from("test_get_documents_for_pid_type");
    let pid = String::from("test_get_documents_for_pid_pid");
    let doc1 = create_test_document(&pid, &dt_id);
    let doc2 = create_test_document(&pid, &dt_id);
    let doc3 = create_test_document(&pid, &dt_id);
    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    // create test data in db
    doc_api.create_document(&TOKEN.to_string(), &doc1)?;
    doc_api.create_document(&TOKEN.to_string(), &doc2)?;
    doc_api.create_document(&TOKEN.to_string(), &doc3)?;

    // run test
    let result = doc_api.get_documents_for_pid(&TOKEN.to_string(), &pid)?;
    println!("Result: {:?}", result);

    // check that we got three documents back
    assert_eq!(result.len(), 3);

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    assert!(doc_api.delete_document(&TOKEN.to_string(), &pid, &doc1.id)?);
    assert!(doc_api.delete_document(&TOKEN.to_string(), &pid, &doc2.id)?);
    assert!(doc_api.delete_document(&TOKEN.to_string(), &pid, &doc3.id)?);


    Ok(())
}

/// Testcase: Ensure that IDS ids can be used if they are url_encoded
#[test]
fn test_create_document_url_encoded_id() -> Result<()>{
    // configure client_api
    let config = util::load_config(CONFIG_FILE);
    let doc_api: DocumentApiClient = util::configure_api(DOCUMENT_API_URL, &config)?;

    // prepare test data
    let dt_id = String::from("test_create_document_url_encoded_id_type_3");
    let pid = String::from("test_create_document_url_encoded_id_process_3");
    let id = String::from("https://w3id.org/idsa/autogen/ResultMessage/71ad9d3a-3743-4966-afa3-f5b02ba91eaa");
    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    let mut doc = create_test_document(&pid, &dt_id);
    doc.id = id.clone();

    // run test
    let hash = doc_api.create_document(&TOKEN.to_string(), &doc);

    // check that it's not an error
    assert!(hash.is_ok());

    // clean up
    assert!(doc_api.delete_document(&TOKEN.to_string(), &doc.pid, &id)?);

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    Ok(())
}