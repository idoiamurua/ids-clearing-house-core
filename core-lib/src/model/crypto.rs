use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct KeyEntry {
    pub id: String,
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl KeyEntry{
    pub fn new(id: String, key: Vec<u8>, nonce: Vec<u8>)-> KeyEntry{
        KeyEntry{
            id,
            key,
            nonce
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct KeyMap {
    pub enc: bool,
    pub keys: HashMap<String, KeyEntry>,
    pub keys_enc: Option<Vec<u8>>,
}

impl KeyMap{
    pub fn new(enc: bool, keys: HashMap<String, KeyEntry>, keys_enc: Option<Vec<u8>>) -> KeyMap{
        KeyMap{
            enc,
            keys,
            keys_enc
        }
    }
 }