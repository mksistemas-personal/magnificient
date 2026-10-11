use shared::entities::audit::audit_fields::AuditFields;
use shared::entities::{Document, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ProducerMarker;

/// Identificador único para a entidade Organização
pub type ProducerId = EntityId<ProducerMarker>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Productor {
    pub id: ProducerId,
    pub name: String,
    pub document: Document,
    pub audit: Option<AuditFields>,
}

impl Productor {
    pub fn new(id: ProducerId, name: impl Into<String>, document: Document) -> Self {
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

    pub fn id(&self) -> ProducerId {
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
    fn test_create_productor() {
        let prod_id = ProducerId::new(TSID::from(300u64));
        let doc = Document::new("20-12345678-6", DocumentType::Cuit);
        let prod = Productor::new(prod_id, "Produtor Agrícola", doc);

        assert_eq!(prod.id(), ProducerId::new(TSID::from(300u64)));
        assert_eq!(prod.name(), "Produtor Agrícola");
        assert_eq!(prod.document().value(), "20-12345678-6");
    }
}
