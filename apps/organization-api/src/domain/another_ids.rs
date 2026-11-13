use shared::entities::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ClientMarker;

pub type ClientId = EntityId<ClientMarker>;
