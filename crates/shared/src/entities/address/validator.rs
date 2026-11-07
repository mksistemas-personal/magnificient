use crate::entities::address::Address;
use crate::entities::address::errors::AddressBusinessErrors;
use crate::errors::AddressError;

pub trait AddressValidator: Send + Sync {
    fn validate(&self, address: &Address) -> Result<(), AddressError>;

    fn sanitize_postal_code(&self, postal_code: &str) -> String {
        postal_code.trim().to_string()
    }

    fn format_postal_code(&self, postal_code: &str) -> String {
        postal_code.trim().to_string()
    }
}

pub fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

pub fn sanitize_digits(val: &str) -> String {
    val.chars().filter(|c| c.is_ascii_digit()).collect()
}

// ==========================================
// Validador Brasil (BR / BRA)
// ==========================================

const BRAZIL_VALID_UFS: [&str; 27] = [
    "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO", "MA", "MT", "MS", "MG", "PA", "PB", "PR",
    "PE", "PI", "RJ", "RN", "RS", "RO", "RR", "SC", "SP", "SE", "TO",
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BrazilAddressValidator;

impl AddressValidator for BrazilAddressValidator {
    fn validate(&self, address: &Address) -> Result<(), AddressError> {
        if is_blank(&address.street) {
            return Err(AddressBusinessErrors::empty_street());
        }
        if is_blank(&address.number) {
            return Err(AddressBusinessErrors::empty_number());
        }
        if is_blank(&address.neighborhood) {
            return Err(AddressBusinessErrors::empty_neighborhood());
        }
        if is_blank(&address.city) {
            return Err(AddressBusinessErrors::empty_city());
        }

        // Validação do Estado / UF
        let uf = address.state_code.trim().to_uppercase();
        if uf.is_empty() {
            if is_blank(&address.state) {
                return Err(AddressBusinessErrors::empty_state_code());
            }
        } else if !BRAZIL_VALID_UFS.contains(&uf.as_str()) {
            return Err(AddressBusinessErrors::invalid_state_code(
                "BR",
                &address.state_code,
            ));
        }

        // Validate the postal code (it must contain 8 digits).
        let digits = self.sanitize_postal_code(&address.postal_code);
        if digits.len() != 8 {
            return Err(AddressBusinessErrors::invalid_postal_code_br());
        }

        Ok(())
    }

    fn sanitize_postal_code(&self, postal_code: &str) -> String {
        sanitize_digits(postal_code)
    }

    fn format_postal_code(&self, postal_code: &str) -> String {
        let digits = self.sanitize_postal_code(postal_code);
        if digits.len() == 8 {
            format!("{}-{}", &digits[0..5], &digits[5..8])
        } else {
            postal_code.trim().to_string()
        }
    }
}

// ==========================================
// Validador Argentina (AR / ARG)
// ==========================================

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ArgentinaAddressValidator;

impl AddressValidator for ArgentinaAddressValidator {
    fn validate(&self, address: &Address) -> Result<(), AddressError> {
        if is_blank(&address.street) {
            return Err(AddressBusinessErrors::empty_street());
        }
        if is_blank(&address.number) {
            return Err(AddressBusinessErrors::empty_number());
        }
        if is_blank(&address.city) {
            return Err(AddressBusinessErrors::empty_city());
        }
        if is_blank(&address.state) && is_blank(&address.state_code) {
            return Err(AddressBusinessErrors::empty_state());
        }

        let postal = address.postal_code.trim();
        if postal.is_empty() {
            return Err(AddressBusinessErrors::empty_postal_code());
        }

        // Código postal argentino:
        // Traditional format: 4 digits (e.g., "1000" to "9999").
        // New CPA format: 1 letter (province) + 4 digits + 3 letters (e.g., "C1024CWN").
        let is_traditional = postal.len() == 4 && postal.chars().all(|c| c.is_ascii_digit());
        let is_cpa = postal.len() == 8
            && postal
                .chars()
                .nth(0)
                .is_some_and(|c| c.is_ascii_alphabetic())
            && postal[1..5].chars().all(|c| c.is_ascii_digit())
            && postal[5..8].chars().all(|c| c.is_ascii_alphabetic());

        if !is_traditional && !is_cpa {
            return Err(AddressBusinessErrors::invalid_postal_code_ar());
        }

        Ok(())
    }

    fn sanitize_postal_code(&self, postal_code: &str) -> String {
        postal_code
            .trim()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_uppercase()
    }

    fn format_postal_code(&self, postal_code: &str) -> String {
        self.sanitize_postal_code(postal_code)
    }
}

// ==========================================
// Validador Estados Unidos (US / USA)
// ==========================================

const US_VALID_STATES: [&str; 56] = [
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
    "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
    "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
    "WI", "WY", "DC", "PR", "VI", "GU", "AS", "MP",
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct UnitedStatesAddressValidator;

impl AddressValidator for UnitedStatesAddressValidator {
    fn validate(&self, address: &Address) -> Result<(), AddressError> {
        if is_blank(&address.street) {
            return Err(AddressBusinessErrors::empty_street());
        }
        if is_blank(&address.city) {
            return Err(AddressBusinessErrors::empty_city());
        }

        let state = address.state_code.trim().to_uppercase();
        if state.is_empty() {
            if is_blank(&address.state) {
                return Err(AddressBusinessErrors::empty_state_code());
            }
        } else if !US_VALID_STATES.contains(&state.as_str()) {
            return Err(AddressBusinessErrors::invalid_state_code(
                "US",
                &address.state_code,
            ));
        }

        let digits = self.sanitize_postal_code(&address.postal_code);
        if digits.len() != 5 && digits.len() != 9 {
            return Err(AddressBusinessErrors::invalid_postal_code_us());
        }

        Ok(())
    }

    fn sanitize_postal_code(&self, postal_code: &str) -> String {
        sanitize_digits(postal_code)
    }

    fn format_postal_code(&self, postal_code: &str) -> String {
        let digits = self.sanitize_postal_code(postal_code);
        if digits.len() == 9 {
            format!("{}-{}", &digits[0..5], &digits[5..9])
        } else if digits.len() == 5 {
            digits
        } else {
            postal_code.trim().to_string()
        }
    }
}

// ==========================================
// Validador Genérico / Outros Países
// ==========================================

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GenericAddressValidator;

impl AddressValidator for GenericAddressValidator {
    fn validate(&self, address: &Address) -> Result<(), AddressError> {
        if is_blank(&address.street) {
            return Err(AddressBusinessErrors::empty_street());
        }
        if is_blank(&address.city) {
            return Err(AddressBusinessErrors::empty_city());
        }
        if is_blank(&address.postal_code) {
            return Err(AddressBusinessErrors::empty_postal_code());
        }
        if is_blank(&address.country_code) && is_blank(&address.country) {
            return Err(AddressBusinessErrors::empty_country_code());
        }

        Ok(())
    }
}

/// Returns the appropriate `AddressValidator` for the country code or name.
pub fn validator_for_country(country_code_or_name: &str) -> Box<dyn AddressValidator> {
    match country_code_or_name.trim().to_uppercase().as_str() {
        "BR" | "BRA" | "BRASIL" | "BRAZIL" => Box::new(BrazilAddressValidator),
        "AR" | "ARG" | "ARGENTINA" => Box::new(ArgentinaAddressValidator),
        "US" | "USA" | "UNITED STATES" | "ESTADOS UNIDOS" => Box::new(UnitedStatesAddressValidator),
        _ => Box::new(GenericAddressValidator),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brazil_validator_success() {
        let validator = BrazilAddressValidator;
        let address = Address::new(
            "Av. Paulista",
            "1000",
            "Apto 101",
            "Bela Vista",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310-100",
            "Brasil",
            "BR",
            "",
        );
        assert!(validator.validate(&address).is_ok());
        assert_eq!(
            validator.sanitize_postal_code(&address.postal_code),
            "01310100"
        );
        assert_eq!(validator.format_postal_code("01310100"), "01310-100");
    }

    #[test]
    fn test_brazil_validator_missing_fields() {
        let validator = BrazilAddressValidator;
        let mut address = Address::new(
            "",
            "1000",
            "",
            "Centro",
            "São Paulo",
            "São Paulo",
            "SP",
            "01310-100",
            "Brasil",
            "BR",
            "",
        );
        assert_eq!(
            validator.validate(&address),
            Err(AddressBusinessErrors::empty_street())
        );

        address.street = "Rua A".to_string();
        address.number = "   ".to_string();
        assert_eq!(
            validator.validate(&address),
            Err(AddressBusinessErrors::empty_number())
        );

        address.number = "10".to_string();
        address.neighborhood = "".to_string();
        assert_eq!(
            validator.validate(&address),
            Err(AddressBusinessErrors::empty_neighborhood())
        );

        address.neighborhood = "Centro".to_string();
        address.state_code = "XX".to_string();
        assert_eq!(
            validator.validate(&address),
            Err(AddressBusinessErrors::invalid_state_code("BR", "XX"))
        );

        address.state_code = "SP".to_string();
        address.postal_code = "123".to_string();
        assert!(matches!(
            validator.validate(&address),
            Err(AddressError::InvalidPostalCode { .. })
        ));
    }

    #[test]
    fn test_argentina_validator() {
        let validator = ArgentinaAddressValidator;
        let valid_traditional = Address::new(
            "Corrientes",
            "1234",
            "",
            "",
            "Buenos Aires",
            "CABA",
            "C",
            "1000",
            "Argentina",
            "AR",
            "",
        );
        assert!(validator.validate(&valid_traditional).is_ok());

        let valid_cpa = Address::new(
            "Corrientes",
            "1234",
            "",
            "",
            "Buenos Aires",
            "CABA",
            "C",
            "C1024CWN",
            "Argentina",
            "AR",
            "",
        );
        assert!(validator.validate(&valid_cpa).is_ok());

        let invalid_cp = Address::new(
            "Corrientes",
            "1234",
            "",
            "",
            "Buenos Aires",
            "CABA",
            "C",
            "INVALID",
            "Argentina",
            "AR",
            "",
        );
        assert!(matches!(
            validator.validate(&invalid_cp),
            Err(AddressError::InvalidPostalCode { .. })
        ));
    }

    #[test]
    fn test_united_states_validator() {
        let validator = UnitedStatesAddressValidator;
        let valid_us = Address::new(
            "123 Main St",
            "Apt 4",
            "",
            "",
            "Springfield",
            "Illinois",
            "IL",
            "62701",
            "United States",
            "US",
            "",
        );
        assert!(validator.validate(&valid_us).is_ok());

        let valid_zip4 = Address::new(
            "123 Main St",
            "",
            "",
            "",
            "Springfield",
            "Illinois",
            "IL",
            "62701-1234",
            "United States",
            "US",
            "",
        );
        assert!(validator.validate(&valid_zip4).is_ok());
        assert_eq!(validator.format_postal_code("627011234"), "62701-1234");

        let invalid_state = Address::new(
            "123 Main St",
            "",
            "",
            "",
            "Springfield",
            "Illinois",
            "ZZ",
            "62701",
            "United States",
            "US",
            "",
        );
        assert_eq!(
            validator.validate(&invalid_state),
            Err(AddressBusinessErrors::invalid_state_code("US", "ZZ"))
        );
    }

    #[test]
    fn test_generic_validator() {
        let validator = GenericAddressValidator;
        let valid_address = Address::new(
            "Gran Via 1",
            "1",
            "",
            "",
            "Madrid",
            "Madrid",
            "MD",
            "28013",
            "Espanha",
            "ES",
            "",
        );
        assert!(validator.validate(&valid_address).is_ok());

        let invalid_address = Address::new(
            "", "", "", "", "Madrid", "Madrid", "MD", "28013", "Espanha", "ES", "",
        );
        assert_eq!(
            validator.validate(&invalid_address),
            Err(AddressBusinessErrors::empty_street())
        );
    }

    #[test]
    fn test_validator_for_country_selection() {
        let br_val = validator_for_country("BR");
        let br_addr = Address::new("A", "1", "", "B", "C", "D", "SP", "01310-100", "", "BR", "");
        assert!(br_val.validate(&br_addr).is_ok());

        let ar_val = validator_for_country("argentina");
        let ar_addr = Address::new("A", "1", "", "", "C", "D", "C", "1000", "", "AR", "");
        assert!(ar_val.validate(&ar_addr).is_ok());

        let us_val = validator_for_country("usa");
        let us_addr = Address::new("A", "", "", "", "C", "D", "CA", "90210", "", "US", "");
        assert!(us_val.validate(&us_addr).is_ok());
    }
}
