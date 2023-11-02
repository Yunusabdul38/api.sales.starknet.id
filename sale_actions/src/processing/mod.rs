use serde_derive::{Deserialize, Serialize};

pub mod purchases;
pub mod renewal;

#[derive(Serialize, Deserialize, Debug)]
pub struct MetadataDoc {
    pub meta_hash: String,
    pub email: String,
    pub tax_state: String,
    pub salt: String,
}
