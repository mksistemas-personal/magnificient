use crate::errors::IdentifierError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
use tsid::TSID;

/// Entidade que encapsula um identificador único TSID fortemente tipado.
///
/// O tipo genérico `T` serve como marcador de tipo (*phantom type*), garantindo em tempo de
/// compilação que identificadores de entidades distintas (ex: `OrganizationId` vs `ClientId`)
/// não sejam misturados acidentalmente.
pub struct EntityId<T> {
    value: TSID,
    _marker: PhantomData<fn() -> T>,
}

impl<T> EntityId<T> {
    /// Cria uma nova instância de `EntityId` a partir de um `TSID`
    pub const fn new(value: TSID) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Cria uma nova instância de `EntityId` validando se o valor é positivo (> 0)
    pub fn new_validated(value: TSID) -> Result<Self, IdentifierError> {
        if value.number() == 0 {
            return Err(IdentifierError::NonPositiveValue(0));
        }
        Ok(Self::new(value))
    }

    /// Retorna o `TSID` do identificador
    pub const fn value(&self) -> TSID {
        self.value
    }

    /// Retorna o `TSID` do identificador
    pub const fn as_tsid(&self) -> TSID {
        self.value
    }

    /// Retorna o valor numérico `u64` do identificador
    pub fn number(&self) -> u64 {
        self.value.number()
    }

    /// Consome o identificador retornando o `TSID`
    pub const fn into_inner(self) -> TSID {
        self.value
    }

    /// Retorna se o identificador possui um valor válido (diferente de zero)
    pub fn is_valid(&self) -> bool {
        self.value.number() > 0
    }
}

// Implementações manuais de traits para não exigir bounds no tipo genérico T

impl<T> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for EntityId<T> {}

impl<T> PartialEq for EntityId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for EntityId<T> {}

impl<T> PartialOrd for EntityId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for EntityId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.number().cmp(&other.value.number())
    }
}

impl<T> Hash for EntityId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Default for EntityId<T> {
    fn default() -> Self {
        Self::new(TSID::from(0u64))
    }
}

impl<T> fmt::Debug for EntityId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntityId({})", self.value)
    }
}

impl<T> fmt::Display for EntityId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> From<TSID> for EntityId<T> {
    fn from(value: TSID) -> Self {
        Self::new(value)
    }
}

impl<T> From<EntityId<T>> for TSID {
    fn from(id: EntityId<T>) -> Self {
        id.value
    }
}

impl<T> Deref for EntityId<T> {
    type Target = TSID;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> AsRef<TSID> for EntityId<T> {
    fn as_ref(&self) -> &TSID {
        &self.value
    }
}

impl<T> FromStr for EntityId<T> {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(IdentifierError::InvalidFormat("String vazia".to_string()));
        }
        if let Ok(tsid) = TSID::try_from(trimmed) {
            return Ok(Self::new(tsid));
        }
        if let Ok(num) = trimmed.parse::<u64>() {
            return Ok(Self::new(TSID::from(num)));
        }
        Err(IdentifierError::ParseError(format!(
            "Não foi possível converter '{s}' em TSID"
        )))
    }
}

impl<T> TryFrom<&str> for EntityId<T> {
    type Error = IdentifierError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl<T> TryFrom<String> for EntityId<T> {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
