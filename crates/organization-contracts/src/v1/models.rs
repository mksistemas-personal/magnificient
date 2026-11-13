use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationDocument {
    pub value: String,
    pub document_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

pub use common_contracts::address::Address as OrganizationAddress;
