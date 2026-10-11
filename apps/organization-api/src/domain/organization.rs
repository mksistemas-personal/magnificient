use shared::entities::audit::audit_fields::AuditFields;
use shared::entities::{Document, EntityId};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OrganizationMarker;

pub type OrganizationId = EntityId<OrganizationMarker>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub document: Document,
    pub audit: Option<AuditFields>,
}

impl Organization {
    pub fn new(id: OrganizationId, name: impl Into<String>, document: Document) -> Self {
        Self {
            id,
            name: name.into(),
            document,
            audit: None,
        }
    }

    pub fn with_audit(mut self, audit: AuditFields) -> Self {
        self.audit = Some(audit);
        self
    }

    pub fn id(&self) -> OrganizationId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn document(&self) -> &Document {
        &self.document
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::entities::{DocumentType, TSID};

    #[test]
    fn test_create_organization() {
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let org = Organization::new(org_id, "Minha Empresa", doc);

        assert_eq!(org.id(), OrganizationId::new(TSID::from(100u64)));
        assert_eq!(org.name(), "Minha Empresa");
        assert_eq!(org.document().value(), "00.000.000/0001-91");
    }
}
