mod add_usecase;
mod all_usecase;
mod by_id_usecase;
mod delete_organization_products_usecase;
mod delete_usecase;
mod persistence;
mod update_usecase;

pub use crate::errors::ProductBusinessErrors;
pub use add_usecase::{CreateProductRequest, CreateProductService};
pub use all_usecase::{FindProductsRequest, FindProductsService};
pub use by_id_usecase::{GetProductByIdRequest, GetProductByIdService};
pub use delete_organization_products_usecase::{
    DeleteOrganizationProductsRequest, DeleteOrganizationProductsService,
};
pub use delete_usecase::{DeleteProductRequest, DeleteProductService};
pub use persistence::ProductPersistence;
pub use update_usecase::{UpdateProductRequest, UpdateProductService};
