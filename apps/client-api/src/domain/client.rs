use shared::entities::{Address, AuditFields, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ClientMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OrganizationMarker;

pub type ClientId = EntityId<ClientMarker>;
pub type OrganizationId = EntityId<OrganizationMarker>;

#[derive(Debug, Clone, PartialEq)]
pub struct Client {
    id: ClientId,
    organization_id: OrganizationId,
    name: String,
    nickname: Option<String>,
    second_address: Option<Address>,
    audit: Option<AuditFields>,
}

impl Client {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ClientId,
        organization_id: OrganizationId,
        name: impl Into<String>,
        nickname: Option<String>,
        second_address: Option<Address>,
    ) -> Self {
        Self {
            id,
            organization_id,
            name: name.into(),
            nickname,
            second_address,
            audit: None,
        }
    }

    pub fn with_audit(mut self, audit: AuditFields) -> Self {
        self.audit = Some(audit);
        self
    }

    pub fn with_second_address(mut self, second_address: Option<Address>) -> Self {
        self.second_address = second_address;
        self
    }

    pub fn id(&self) -> ClientId {
        self.id
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }

    pub fn second_address(&self) -> Option<&Address> {
        self.second_address.as_ref()
    }

    pub fn set_second_address(&mut self, second_address: Option<Address>) {
        self.second_address = second_address;
    }

    pub fn audit(&self) -> Option<&AuditFields> {
        self.audit.as_ref()
    }

    pub fn update(
        &mut self,
        organization_id: OrganizationId,
        name: impl Into<String>,
        nickname: Option<String>,
        second_address: Option<Address>,
        updated_by: &str,
    ) {
        self.organization_id = organization_id;
        self.name = name.into();
        self.nickname = nickname;
        self.second_address = second_address;
        if let Some(audit) = self.audit.as_mut() {
            audit.updated(updated_by);
        } else {
            self.audit = Some(AuditFields::created(updated_by));
        }
    }
}
