use shared::entities::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ClientMarker;

pub type ClientId = EntityId<ClientMarker>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SupplierMarker;

pub type SupplierId = EntityId<SupplierMarker>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ProductMarker;

pub type ProductId = EntityId<ProductMarker>;
