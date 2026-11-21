use shared::entities::{AuditFields, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ProductMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OrganizationMarker;

pub type ProductId = EntityId<ProductMarker>;
pub type OrganizationId = EntityId<OrganizationMarker>;

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    id: ProductId,
    organization_id: OrganizationId,
    name: String,
    description: Option<String>,
    sku: Option<String>,
    price: Option<f64>,
    audit: Option<AuditFields>,
}

impl Product {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ProductId,
        organization_id: OrganizationId,
        name: impl Into<String>,
        description: Option<String>,
        sku: Option<String>,
        price: Option<f64>,
    ) -> Self {
        Self {
            id,
            organization_id,
            name: name.into(),
            description,
            sku,
            price,
            audit: None,
        }
    }

    pub fn with_audit(mut self, audit: AuditFields) -> Self {
        self.audit = Some(audit);
        self
    }

    pub fn id(&self) -> ProductId {
        self.id
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn sku(&self) -> Option<&str> {
        self.sku.as_deref()
    }

    pub fn price(&self) -> Option<f64> {
        self.price
    }

    pub fn audit(&self) -> Option<&AuditFields> {
        self.audit.as_ref()
    }

    pub fn update(
        &mut self,
        organization_id: OrganizationId,
        name: impl Into<String>,
        description: Option<String>,
        sku: Option<String>,
        price: Option<f64>,
        updated_by: &str,
    ) {
        self.organization_id = organization_id;
        self.name = name.into();
        self.description = description;
        self.sku = sku;
        self.price = price;
        if let Some(audit) = self.audit.as_mut() {
            audit.updated(updated_by);
        } else {
            self.audit = Some(AuditFields::created(updated_by));
        }
    }
}
