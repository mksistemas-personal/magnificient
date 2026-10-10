use shared::entities::use_case::business_error::BusinessError;

/// Entidade que centraliza as constantes e geradores de `BusinessError` para o domínio de Organização,
/// padronizando as mensagens e erros de negócio utilizados nos casos de uso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OrganizationBusinessErrors;

impl OrganizationBusinessErrors {
    /// Mensagem de erro para organização não encontrada.
    pub const ORG_NOT_FOUND: &'static str = "organizatgion.not.found";

    /// Mensagem de erro para nome da organização em branco ou vazio.
    pub const ORG_NAME_EMPTY: &'static str = "organization.name.empty";

    /// Mensagem de erro para nome da organização que ultrapassa o tamanho máximo permitido.
    pub const ORG_NAME_TOO_LONG: &'static str = "organization.name.greather.limit";

    /// Mensagem de erro para organização já existente / duplicada.
    pub const ORG_ALREADY_EXISTS: &'static str = "organization.already.exists";

    /// Mensagem de erro para documento fiscal da organização inválido.
    pub const ORG_DOCUMENT_INVALID: &'static str = "organization.document.invalid";

    /// Mensagem de erro para documento fiscal já em uso por outra organização.
    pub const ORG_DOCUMENT_ALREADY_IN_USE: &'static str = "organization.document.already.in.use";

    /// Mensagem de erro para identificador da organização inválido.
    pub const ORG_ID_INVALID: &'static str = "organization.id.invalid";

    /// Mensagem de erro para ausência de autor na criação da organização.
    pub const ORG_CREATOR_REQUIRED: &'static str = "organization.creator.required";

    /// Mensagem de erro para operações em organizações com cadastro inativo.
    pub const ORG_INACTIVE: &'static str = "organization.inactive";

    // --- Métodos de Conveniência para Criar Instâncias de BusinessError ---

    /// Retorna `BusinessError::NotFound` para uma organização pelo ID.
    pub fn not_found() -> BusinessError {
        BusinessError::not_found(Self::ORG_NOT_FOUND)
    }

    /// Retorna `BusinessError::InvalidData` quando o nome está vazio.
    pub fn name_empty() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_NAME_EMPTY)
    }

    /// Retorna `BusinessError::InvalidData` quando o nome excede o tamanho limite.
    pub fn name_too_long() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_NAME_TOO_LONG)
    }

    /// Retorna `BusinessError::AlreadyExists` quando a organização já existe.
    pub fn already_exists() -> BusinessError {
        BusinessError::already_exists(Self::ORG_ALREADY_EXISTS)
    }

    /// Retorna `BusinessError::InvalidData` quando o documento é inválido.
    pub fn document_invalid() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_DOCUMENT_INVALID)
    }

    /// Retorna `BusinessError::AlreadyExists` quando o documento já está cadastrado.
    pub fn document_already_in_use() -> BusinessError {
        BusinessError::already_exists(Self::ORG_DOCUMENT_ALREADY_IN_USE)
    }

    /// Retorna `BusinessError::InvalidData` quando o ID é inválido.
    pub fn id_invalid() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_ID_INVALID)
    }

    /// Retorna `BusinessError::InvalidData` quando o criador é obrigatório.
    pub fn creator_required() -> BusinessError {
        BusinessError::invalid_data(Self::ORG_CREATOR_REQUIRED)
    }

    /// Retorna `BusinessError::NotAllowed` quando a organização está inativa.
    pub fn inactive() -> BusinessError {
        BusinessError::not_allowed(Self::ORG_INACTIVE)
    }
}

/// Alias para conveniência
pub type OrganizationErrors = OrganizationBusinessErrors;
