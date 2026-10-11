pub mod entity_id;

pub use entity_id::EntityId;
pub use tsid::TSID;

/// Alias simplificado para `EntityId<T>`
pub type Id<T> = EntityId<T>;

#[macro_export]
macro_rules! define_id {
    ($id_name:ident, $marker_name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $marker_name;

        pub type $id_name = $crate::entities::identifier::EntityId<$marker_name>;
    };
    ($id_name:ident) => {
        paste::paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
            pub struct [<$id_name Marker>];

            pub type $id_name = $crate::entities::identifier::EntityId<[<$id_name Marker>]>;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tsid::TSID;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct EntityTestMarker;

    type EntityTestId = EntityId<EntityTestMarker>;

    #[test]
    fn test_id_creation_and_values() {
        let tsid = TSID::from(1001u64);
        let org_id = EntityTestId::new(tsid);
        assert_eq!(org_id.value(), tsid);
        assert_eq!(org_id.as_tsid(), tsid);
        assert_eq!(org_id.number(), 1001);
        assert_eq!(*org_id, tsid);
        assert!(org_id.is_valid());

        let raw_val: TSID = org_id.into();
        assert_eq!(raw_val, tsid);
    }

    #[test]
    fn test_id_validation() {
        let valid_tsid = TSID::from(42u64);
        let valid_id = EntityTestId::new_validated(valid_tsid);
        assert!(valid_id.is_ok());
        assert_eq!(valid_id.unwrap().value(), valid_tsid);

        let invalid_zero = EntityTestId::new_validated(TSID::from(0u64));
        assert!(invalid_zero.is_err());
    }

    #[test]
    fn test_display_and_parsing() {
        let tsid = TSID::from(987654321u64);
        let prod_id = EntityTestId::new(tsid);
        let formatted = format!("{prod_id}");
        assert_eq!(formatted, tsid.to_string());

        let parsed: EntityTestId = formatted.parse().expect("Deveria converter com sucesso");
        assert_eq!(parsed, prod_id);

        let parsed_numeric: EntityTestId = "987654321".parse().expect("Deveria converter numérico");
        assert_eq!(parsed_numeric, prod_id);

        let invalid_parse: Result<EntityTestId, _> = "invalid-tsid!".parse();
        assert!(invalid_parse.is_err());
    }

    #[test]
    fn test_hash_and_equality() {
        let id1 = EntityTestId::new(TSID::from(10u64));
        let id2 = EntityTestId::new(TSID::from(10u64));
        let id3 = EntityTestId::new(TSID::from(20u64));

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2));
        assert!(!set.contains(&id3));
    }

    #[test]
    fn test_macro_defined_id() {
        define_id!(CustomOrderId, CustomOrderMarker);
        let tsid = TSID::from(55u64);
        let order_id = CustomOrderId::new(tsid);
        assert_eq!(order_id.value(), tsid);
        assert_eq!(order_id.number(), 55);
    }
}
