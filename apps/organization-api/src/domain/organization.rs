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

    pub fn update(&mut self, name: impl Into<String>, updated_by: impl Into<String>) -> &mut Self {
        self.name = name.into();
        let updated_by = updated_by.into();
        if let Some(audit) = self.audit.as_mut() {
            audit.updated(updated_by);
        } else {
            self.audit = Some(AuditFields::created(updated_by));
        }
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

    #[test]
    fn test_update_organization_with_existing_audit() {
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let mut org = Organization::new(org_id, "Nome Antigo", doc)
            .with_audit(AuditFields::created("admin"));

        org.update("Novo Nome", "editor");

        assert_eq!(org.name(), "Novo Nome");
        let audit = org.audit.as_ref().unwrap();
        assert_eq!(audit.created_by(), "admin");
        assert_eq!(audit.updated_by(), "editor");
    }

    #[test]
    fn test_update_organization_without_existing_audit() {
        let org_id = OrganizationId::new(TSID::from(100u64));
        let doc = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        let mut org = Organization::new(org_id, "Nome Antigo", doc);

        org.update("Novo Nome", "editor");

        assert_eq!(org.name(), "Novo Nome");
        let audit = org.audit.as_ref().unwrap();
        assert_eq!(audit.created_by(), "editor");
        assert_eq!(audit.updated_by(), "editor");
    }
}
