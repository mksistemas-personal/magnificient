use serde::{Deserialize, Serialize};

pub const EVENT_NAME: &str = "ProductEvent";
/// Preserves the version value currently published in the AMQP header.
pub const EVENT_VERSION: i32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProductEventType {
    AddProduct,
    UpdateProduct,
    DeleteProduct,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductEvent {
    pub event_type: ProductEventType,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
}

impl ProductEvent {
    pub fn add(
        id: impl Into<String>,
        organization_id: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        sku: Option<String>,
        price: Option<f64>,
    ) -> Self {
        Self {
            event_type: ProductEventType::AddProduct,
            id: id.into(),
            organization_id: Some(organization_id.into()),
            name: Some(name.into()),
            description,
            sku,
            price,
        }
    }

    pub fn update(
        id: impl Into<String>,
        organization_id: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        sku: Option<String>,
        price: Option<f64>,
    ) -> Self {
        Self {
            event_type: ProductEventType::UpdateProduct,
            id: id.into(),
            organization_id: Some(organization_id.into()),
            name: Some(name.into()),
            description,
            sku,
            price,
        }
    }

    pub fn delete(id: impl Into<String>) -> Self {
        Self {
            event_type: ProductEventType::DeleteProduct,
            id: id.into(),
            organization_id: None,
            name: None,
            description: None,
            sku: None,
            price: None,
        }
    }
}
