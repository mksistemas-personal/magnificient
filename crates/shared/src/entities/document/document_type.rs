use crate::entities::document::validator;
use crate::errors::DocumentError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Country {
    Brazil,
    Argentina,
    Other(String),
}

impl Country {
    pub fn from_code(code: &str) -> Self {
        match code.trim().to_uppercase().as_str() {
            "BR" => Country::Brazil,
            "AR" => Country::Argentina,
            other => Country::Other(other.to_string()),
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Country::Brazil => "BR",
            Country::Argentina => "AR",
            Country::Other(code) => code.as_str(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Country::Brazil => "Brasil",
            Country::Argentina => "Argentina",
            Country::Other(name) => name.as_str(),
        }
    }
}

impl From<&str> for Country {
    fn from(code: &str) -> Self {
        Self::from_code(code)
    }
}

impl From<String> for Country {
    fn from(code: String) -> Self {
        Self::from_code(&code)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocumentType {
    /// Cadastro Nacional da Pessoa Jurídica (Brasil)
    Cnpj,
    /// Cadastro de Pessoas Físicas (Brasil)
    Cpf,
    /// Código Único de Identificación Tributaria (Argentina)
    Cuit,
    /// Código Único de Identificación Laboral (Argentina)
    Cuil,
    /// Documento Nacional de Identidad (Argentina)
    Dni,
    /// Passaporte
    Passport,
    /// Tipo de documento customizado / outros países
    Other {
        name: String,
        country: Option<Country>,
    },
}

impl DocumentType {
    /// Constrói um `DocumentType` a partir de um código textual e país opcional.
    pub fn from_code(code: &str, country: Option<Country>) -> Self {
        match code.trim().to_uppercase().as_str() {
            "CNPJ" => DocumentType::Cnpj,
            "CPF" => DocumentType::Cpf,
            "CUIT" => DocumentType::Cuit,
            "CUIL" => DocumentType::Cuil,
            "DNI" => DocumentType::Dni,
            "PASSPORT" | "PASSAPORTE" => DocumentType::Passport,
            other => DocumentType::Other {
                name: other.to_string(),
                country,
            },
        }
    }

    /// Retorna o país associado ao tipo de documento, se houver
    pub fn country(&self) -> Option<Country> {
        match self {
            DocumentType::Cnpj | DocumentType::Cpf => Some(Country::Brazil),
            DocumentType::Cuit | DocumentType::Cuil | DocumentType::Dni => Some(Country::Argentina),
            DocumentType::Passport => None,
            DocumentType::Other { country, .. } => country.clone(),
        }
    }

    /// Retorna o identificador/código textual do tipo de documento
    pub fn code(&self) -> &str {
        match self {
            DocumentType::Cnpj => "CNPJ",
            DocumentType::Cpf => "CPF",
            DocumentType::Cuit => "CUIT",
            DocumentType::Cuil => "CUIL",
            DocumentType::Dni => "DNI",
            DocumentType::Passport => "PASSPORT",
            DocumentType::Other { name, .. } => name.as_str(),
        }
    }

    /// Retorna uma descrição amigável do documento
    pub fn description(&self) -> &str {
        match self {
            DocumentType::Cnpj => "Cadastro Nacional da Pessoa Jurídica (Brasil)",
            DocumentType::Cpf => "Cadastro de Pessoas Físicas (Brasil)",
            DocumentType::Cuit => "Código Único de Identificación Tributaria (Argentina)",
            DocumentType::Cuil => "Código Único de Identificación Laboral (Argentina)",
            DocumentType::Dni => "Documento Nacional de Identidad (Argentina)",
            DocumentType::Passport => "Passaporte",
            DocumentType::Other { name, .. } => name.as_str(),
        }
    }

    /// Executa a validação do valor de acordo com as regras específicas deste tipo de documento
    pub fn validate(&self, value: &str) -> Result<(), DocumentError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(DocumentError::EmptyDocument);
        }

        match self {
            DocumentType::Cnpj => validator::validate_cnpj(trimmed),
            DocumentType::Cpf => validator::validate_cpf(trimmed),
            DocumentType::Cuit | DocumentType::Cuil => validator::validate_cuit(trimmed),
            DocumentType::Dni => validator::validate_dni(trimmed),
            DocumentType::Passport => {
                if trimmed.len() < 3 {
                    Err(DocumentError::InvalidLength {
                        expected: 3,
                        received: trimmed.len(),
                    })
                } else {
                    Ok(())
                }
            }
            DocumentType::Other { .. } => Ok(()),
        }
    }

    /// Formata a string do documento segundo o padrão do tipo
    pub fn format(&self, value: &str) -> String {
        match self {
            DocumentType::Cnpj => validator::format_cnpj(value),
            DocumentType::Cpf => validator::format_cpf(value),
            DocumentType::Cuit | DocumentType::Cuil => validator::format_cuit(value),
            DocumentType::Dni => validator::format_dni(value),
            _ => value.trim().to_string(),
        }
    }

    /// Higieniza o valor do documento removendo caracteres de pontuação quando aplicável
    pub fn sanitize(&self, value: &str) -> String {
        match self {
            DocumentType::Cnpj
            | DocumentType::Cpf
            | DocumentType::Cuit
            | DocumentType::Cuil
            | DocumentType::Dni => validator::sanitize_digits(value),
            _ => value.trim().to_string(),
        }
    }
}

impl From<&str> for DocumentType {
    fn from(code: &str) -> Self {
        Self::from_code(code, None)
    }
}

impl From<String> for DocumentType {
    fn from(code: String) -> Self {
        Self::from_code(&code, None)
    }
}

impl From<(&str, Option<Country>)> for DocumentType {
    fn from((code, country): (&str, Option<Country>)) -> Self {
        Self::from_code(code, country)
    }
}

impl From<(String, Option<Country>)> for DocumentType {
    fn from((code, country): (String, Option<Country>)) -> Self {
        Self::from_code(&code, country)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_from_code() {
        assert_eq!(Country::from_code("BR"), Country::Brazil);
        assert_eq!(Country::from_code("br"), Country::Brazil);
        assert_eq!(Country::from_code(" BR "), Country::Brazil);
        assert_eq!(Country::from_code("AR"), Country::Argentina);
        assert_eq!(Country::from_code("ar"), Country::Argentina);
        assert_eq!(Country::from_code("US"), Country::Other("US".to_string()));
        assert_eq!(Country::from_code("us"), Country::Other("US".to_string()));
    }

    #[test]
    fn test_country_from_traits() {
        let from_str: Country = Country::from("BR");
        assert_eq!(from_str, Country::Brazil);

        let from_string: Country = Country::from("ar".to_string());
        assert_eq!(from_string, Country::Argentina);

        let from_other: Country = Country::from("CL");
        assert_eq!(from_other, Country::Other("CL".to_string()));
    }

    #[test]
    fn test_country_code_and_name() {
        assert_eq!(Country::Brazil.code(), "BR");
        assert_eq!(Country::Brazil.name(), "Brasil");
        assert_eq!(Country::Argentina.code(), "AR");
        assert_eq!(Country::Argentina.name(), "Argentina");
        let other = Country::Other("CL".to_string());
        assert_eq!(other.code(), "CL");
        assert_eq!(other.name(), "CL");
    }

    #[test]
    fn test_document_type_from_code() {
        assert_eq!(DocumentType::from_code("CNPJ", None), DocumentType::Cnpj);
        assert_eq!(DocumentType::from_code("cnpj", None), DocumentType::Cnpj);
        assert_eq!(DocumentType::from_code(" CPF ", None), DocumentType::Cpf);
        assert_eq!(DocumentType::from_code("cuit", None), DocumentType::Cuit);
        assert_eq!(DocumentType::from_code("CUIL", None), DocumentType::Cuil);
        assert_eq!(DocumentType::from_code("dni", None), DocumentType::Dni);
        assert_eq!(DocumentType::from_code("PASSPORT", None), DocumentType::Passport);
        assert_eq!(DocumentType::from_code("passaporte", None), DocumentType::Passport);
        assert_eq!(
            DocumentType::from_code("TAX_ID", Some(Country::Brazil)),
            DocumentType::Other {
                name: "TAX_ID".to_string(),
                country: Some(Country::Brazil),
            }
        );
    }

    #[test]
    fn test_document_type_from_traits() {
        let from_str: DocumentType = DocumentType::from("CNPJ");
        assert_eq!(from_str, DocumentType::Cnpj);

        let from_string: DocumentType = DocumentType::from("cpf".to_string());
        assert_eq!(from_string, DocumentType::Cpf);

        let from_tuple_str: DocumentType = DocumentType::from(("CUSTOM", Some(Country::Argentina)));
        assert_eq!(
            from_tuple_str,
            DocumentType::Other {
                name: "CUSTOM".to_string(),
                country: Some(Country::Argentina),
            }
        );

        let from_tuple_string: DocumentType =
            DocumentType::from(("CUSTOM".to_string(), Some(Country::Brazil)));
        assert_eq!(
            from_tuple_string,
            DocumentType::Other {
                name: "CUSTOM".to_string(),
                country: Some(Country::Brazil),
            }
        );
    }
}
