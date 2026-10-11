use crate::entities::document::validator;
use crate::errors::DocumentError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Country {
    Brazil,
    Argentina,
    Other(String),
}

impl Country {
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
