use shared::entities::audit::audit_fields::AuditFields;
use shared::entities::{Address, EntityId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ClientMarker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OrganizationMarker;

pub type ClientId = EntityId<ClientMarker>;
pub type OrganizationId = EntityId<OrganizationMarker>;

#[derive(Debug, Clone, PartialEq)]
pub struct Client {
    pub id: ClientId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub nickname: Option<String>,
    pub second_address: Option<Address>,
    pub audit: Option<AuditFields>,
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

    pub fn id(&self) -> ClientId {
        self.id
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }

    pub fn name(&self) -> &str {
        &self.name
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
