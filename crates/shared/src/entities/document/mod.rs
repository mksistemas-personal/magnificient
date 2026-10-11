pub mod document_type;
pub mod validator;

pub use document_type::{Country, DocumentType};

use crate::errors::DocumentError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub value: String,
    pub document_type: DocumentType,
}

impl Document {
    /// Cria uma nova instância de Documento sem validação prévia
    pub fn new(value: impl Into<String>, document_type: DocumentType) -> Self {
        Self {
            value: value.into(),
            document_type,
        }
    }

    /// Cria uma nova instância de Documento validando o valor de acordo com as regras do tipo
    pub fn new_validated(
        value: impl Into<String>,
        document_type: DocumentType,
    ) -> Result<Self, DocumentError> {
        let doc = Self::new(value, document_type);
        doc.validate()?;
        Ok(doc)
    }

    /// Executa a validação das regras de negócio do tipo de documento
    pub fn validate(&self) -> Result<(), DocumentError> {
        self.document_type.validate(&self.value)
    }

    /// Retorna se o documento é válido
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Retorna o valor do documento sanitizado (apenas números para documentos numéricos)
    pub fn sanitized_value(&self) -> String {
        self.document_type.sanitize(&self.value)
    }

    /// Retorna o valor formatado com a pontuação padrão do documento
    pub fn formatted(&self) -> String {
        self.document_type.format(&self.value)
    }

    /// Retorna o país de origem do documento, se aplicável
    pub fn country(&self) -> Option<Country> {
        self.document_type.country()
    }

    /// Retorna uma referência ao tipo do documento
    pub fn document_type(&self) -> &DocumentType {
        &self.document_type
    }

    /// Retorna uma referência ao valor bruto do documento
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cnpj() {
        // CNPJ válido de teste (ex: Banco do Brasil)
        let cnpj = Document::new("00.000.000/0001-91", DocumentType::Cnpj);
        assert!(cnpj.is_valid());
        assert_eq!(cnpj.country(), Some(Country::Brazil));
        assert_eq!(cnpj.sanitized_value(), "00000000000191");
        assert_eq!(cnpj.formatted(), "00.000.000/0001-91");
    }

    #[test]
    fn test_invalid_cnpj_check_digits() {
        let cnpj = Document::new("00.000.000/0001-00", DocumentType::Cnpj);
        assert!(!cnpj.is_valid());
        assert_eq!(cnpj.validate(), Err(DocumentError::InvalidCheckDigits));
    }

    #[test]
    fn test_invalid_cnpj_repeated_digits() {
        let cnpj = Document::new("11.111.111/1111-11", DocumentType::Cnpj);
        assert!(!cnpj.is_valid());
        assert_eq!(cnpj.validate(), Err(DocumentError::InvalidCheckDigits));
    }

    #[test]
    fn test_valid_cuit_argentina() {
        // CUIT válido de exemplo argentino
        let cuit = Document::new("20-12345678-6", DocumentType::Cuit);
        assert!(cuit.is_valid());
        assert_eq!(cuit.country(), Some(Country::Argentina));
        assert_eq!(cuit.sanitized_value(), "20123456786");
        assert_eq!(cuit.formatted(), "20-12345678-6");
    }

    #[test]
    fn test_invalid_cuit_argentina() {
        let cuit = Document::new("20-12345678-0", DocumentType::Cuit);
        assert!(!cuit.is_valid());
        assert_eq!(cuit.validate(), Err(DocumentError::InvalidCheckDigits));
    }

    #[test]
    fn test_valid_cpf() {
        let cpf = Document::new("123.456.789-09", DocumentType::Cpf);
        assert!(cpf.is_valid());
        assert_eq!(cpf.country(), Some(Country::Brazil));
        assert_eq!(cpf.sanitized_value(), "12345678909");
    }

    #[test]
    fn test_valid_dni_argentina() {
        let dni = Document::new("12.345.678", DocumentType::Dni);
        assert!(dni.is_valid());
        assert_eq!(dni.country(), Some(Country::Argentina));
        assert_eq!(dni.sanitized_value(), "12345678");
    }

    #[test]
    fn test_new_validated_error() {
        let result = Document::new_validated("123", DocumentType::Cnpj);
        assert!(result.is_err());
    }
}
