use shared::entities::audit::audit_fields::AuditFields;
use shared::entities::{Document, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ClientMarker;

pub type ClientId = EntityId<ClientMarker>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
    pub document: Document,
    pub audit: Option<AuditFields>,
}

impl Client {
    pub fn new(id: ClientId, name: impl Into<String>, document: Document) -> Self {
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

    pub fn id(&self) -> ClientId {
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
    fn test_create_client() {
        let client_id = ClientId::new(TSID::from(200u64));
        let doc = Document::new("123.456.789-09", DocumentType::Cpf);
        let client = Client::new(client_id, "João Silva", doc);

        assert_eq!(client.id(), ClientId::new(TSID::from(200u64)));
        assert_eq!(client.name(), "João Silva");
        assert_eq!(client.document().value(), "123.456.789-09");
    }
}
