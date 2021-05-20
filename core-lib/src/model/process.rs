#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Process {
    pub id: String,
    pub owner: String,
}

impl Process {
    pub fn new(id: String, owner: String) -> Process {
        Process {
            id,
            owner
        }
    }
}