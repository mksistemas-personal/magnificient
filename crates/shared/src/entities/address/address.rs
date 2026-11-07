use serde::{Deserialize, Serialize};
use tsid::TSID;

use crate::entities::address::validator::{AddressValidator, validator_for_country};
use crate::entities::audit::audit_fields::AuditFields;
use crate::errors::AddressError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Address {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<TSID>,
    pub street: String,
    pub number: String,
    pub complement: String,
    pub neighborhood: String,
    pub city: String,
    pub state: String,
    pub state_code: String,
    pub postal_code: String,
    pub country: String,
    pub country_code: String,
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditFields>,
}

impl Address {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        street: impl Into<String>,
        number: impl Into<String>,
        complement: impl Into<String>,
        neighborhood: impl Into<String>,
        city: impl Into<String>,
        state: impl Into<String>,
        state_code: impl Into<String>,
        postal_code: impl Into<String>,
        country: impl Into<String>,
        country_code: impl Into<String>,
        reference: impl Into<String>,
    ) -> Self {
        Self {
            id: None,
            street: street.into(),
            number: number.into(),
            complement: complement.into(),
            neighborhood: neighborhood.into(),
            city: city.into(),
            state: state.into(),
            state_code: state_code.into(),
            postal_code: postal_code.into(),
            country: country.into(),
            country_code: country_code.into(),
            reference: reference.into(),
            audit: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_validated(
        street: impl Into<String>,
        number: impl Into<String>,
        complement: impl Into<String>,
        neighborhood: impl Into<String>,
        city: impl Into<String>,
        state: impl Into<String>,
        state_code: impl Into<String>,
        postal_code: impl Into<String>,
        country: impl Into<String>,
        country_code: impl Into<String>,
        reference: impl Into<String>,
    ) -> Result<Self, AddressError> {
        let address = Self::new(
            street,
            number,
            complement,
            neighborhood,
            city,
            state,
            state_code,
            postal_code,
            country,
            country_code,
            reference,
        );
        address.validate()?;
        Ok(address)
    }

    pub fn with_id(mut self, id: TSID) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_audit(mut self, audit: AuditFields) -> Self {
        self.audit = Some(audit);
        self
    }

    pub fn id(&self) -> Option<TSID> {
        self.id
    }

    pub fn set_id(&mut self, id: TSID) -> &mut Self {
        self.id = Some(id);
        self
    }

    pub fn audit(&self) -> Option<&AuditFields> {
        self.audit.as_ref()
    }

    pub fn set_audit(&mut self, audit: Option<AuditFields>) -> &mut Self {
        self.audit = audit;
        self
    }

    /// Returns the appropriate validator for the country code or name.
    pub fn validator(&self) -> Box<dyn AddressValidator> {
        if !self.country_code.trim().is_empty() {
            validator_for_country(&self.country_code)
        } else {
            validator_for_country(&self.country)
        }
    }

    /// Runs the address validation rules.
    pub fn validate(&self) -> Result<(), AddressError> {
        self.validator().validate(self)
    }

    /// Returns whether the address is valid.
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Returns the postal code sanitized according to the country format.
    pub fn sanitized_postal_code(&self) -> String {
        self.validator().sanitize_postal_code(&self.postal_code)
    }

    /// Returns the postal code formatted according to the country format.
    pub fn formatted_postal_code(&self) -> String {
        self.validator().format_postal_code(&self.postal_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_address() {
        let address = Address::new(
            "Av. Paulista",
            "1000",
            "Conjunto 101",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310-100",
            "Brasil",
            "BR",
            "Próximo ao metrô",
        );

        assert_eq!(address.street, "Av. Paulista");
        assert_eq!(address.number, "1000");
        assert_eq!(address.complement, "Conjunto 101");
        assert_eq!(address.neighborhood, "Bela Vista");
        assert_eq!(address.city, "São Paulo");
        assert_eq!(address.state, "São Paulo");
        assert_eq!(address.state_code, "SP");
        assert_eq!(address.postal_code, "01310-100");
        assert_eq!(address.country, "Brasil");
        assert_eq!(address.country_code, "BR");
        assert_eq!(address.reference, "Próximo ao metrô");
        assert!(address.is_valid());
        assert_eq!(address.sanitized_postal_code(), "01310100");
        assert_eq!(address.formatted_postal_code(), "01310-100");
    }

    #[test]
    fn test_address_new_validated_success() {
        let address = Address::new_validated(
            "Av. Paulista",
            "1000",
            "Conjunto 101",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310100",
            "Brasil",
            "BR",
            "",
        );
        assert!(address.is_ok());
        let addr = address.unwrap();
        assert_eq!(addr.formatted_postal_code(), "01310-100");
    }

    #[test]
    fn test_address_new_validated_error() {
        let result = Address::new_validated(
            "",
            "1000",
            "",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310100",
            "Brasil",
            "BR",
            "",
        );
        assert_eq!(
            result,
            Err(crate::entities::address::AddressBusinessErrors::empty_street())
        );
    }
}
