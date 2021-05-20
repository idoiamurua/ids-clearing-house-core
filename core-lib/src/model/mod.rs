pub mod crypto;
pub mod document;
pub mod process;

#[cfg(test)] mod tests;

pub fn new_uuid() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_hyphenated().to_string()
}