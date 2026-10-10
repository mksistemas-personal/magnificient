use chrono::{DateTime, Utc};

/// Entidade que encapsula os campos de auditoria de registros do sistema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFields {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
}

impl AuditFields {
    /// Cria uma nova instância de `AuditFields` inicializando os campos de criação e
    /// replicando os mesmos valores nos campos de atualização.
    ///
    /// # Exemplo
    /// ```
    /// use shared::entities::AuditFields;
    ///
    /// let audit = AuditFields::created("admin");
    /// assert_eq!(audit.created_by, "admin");
    /// assert_eq!(audit.updated_by, "admin");
    /// assert_eq!(audit.created_at, audit.updated_at);
    /// ```
    pub fn created(created_by: impl Into<String>) -> Self {
        let now = Utc::now();
        let user = created_by.into();
        Self {
            created_at: now,
            updated_at: now,
            created_by: user.clone(),
            updated_by: user,
        }
    }

    /// Cria uma nova instância de `AuditFields` com data/hora e autor específicos de criação,
    /// atribuindo-os também aos campos de atualização.
    pub fn created_with_time(created_by: impl Into<String>, created_at: DateTime<Utc>) -> Self {
        let user = created_by.into();
        Self {
            created_at,
            updated_at: created_at,
            created_by: user.clone(),
            updated_by: user,
        }
    }

    /// Atualiza apenas os campos de atualização (`updated_by` e `updated_at`),
    /// mantendo intactos os campos originais de criação (`created_by` e `created_at`).
    ///
    /// O campo `updated_at` é automaticamente definido com o instante atual em UTC.
    ///
    /// # Exemplo
    /// ```
    /// use shared::entities::AuditFields;
    ///
    /// let mut audit = AuditFields::created("admin");
    /// audit.updated("editor");
    /// assert_eq!(audit.created_by, "admin");
    /// assert_eq!(audit.updated_by, "editor");
    /// ```
    pub fn updated(&mut self, updated_by: impl Into<String>) {
        self.updated_at = Utc::now();
        self.updated_by = updated_by.into();
    }

    /// Atualiza os campos de atualização com autor e data/hora específicos.
    pub fn updated_with_time(&mut self, updated_by: impl Into<String>, updated_at: DateTime<Utc>) {
        self.updated_at = updated_at;
        self.updated_by = updated_by.into();
    }

    /// Alias para o método `updated`.
    pub fn update(&mut self, updated_by: impl Into<String>) {
        self.updated(updated_by);
    }

    /// Retorna uma nova instância atualizada com os novos dados de atualização (estilo builder).
    pub fn with_updated(mut self, updated_by: impl Into<String>) -> Self {
        self.updated(updated_by);
        self
    }

    /// Retorna o autor da criação.
    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    /// Retorna o autor da última atualização.
    pub fn updated_by(&self) -> &str {
        &self.updated_by
    }

    /// Retorna o timestamp de criação.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Retorna o timestamp da última atualização.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Verifica se a entidade foi modificada após a criação.
    pub fn is_modified(&self) -> bool {
        self.created_at != self.updated_at || self.created_by != self.updated_by
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    #[test]
    fn test_audit_created() {
        let audit = AuditFields::created("user_admin");

        assert_eq!(audit.created_by(), "user_admin");
        assert_eq!(audit.updated_by(), "user_admin");
        assert_eq!(audit.created_at(), audit.updated_at());
        assert!(!audit.is_modified());
    }

    #[test]
    fn test_audit_created_with_time() {
        let timestamp = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        let audit = AuditFields::created_with_time("system", timestamp);

        assert_eq!(audit.created_by, "system");
        assert_eq!(audit.updated_by, "system");
        assert_eq!(audit.created_at, timestamp);
        assert_eq!(audit.updated_at, timestamp);
        assert!(!audit.is_modified());
    }

    #[test]
    fn test_audit_updated() {
        let initial_time = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let mut audit = AuditFields::created_with_time("creator_user", initial_time);

        let update_time = initial_time + Duration::hours(2);
        audit.updated_with_time("editor_user", update_time);

        // Campos de criação devem permanecer inalterados
        assert_eq!(audit.created_by, "creator_user");
        assert_eq!(audit.created_at, initial_time);

        // Campos de atualização devem refletir os novos valores
        assert_eq!(audit.updated_by, "editor_user");
        assert_eq!(audit.updated_at, update_time);
        assert!(audit.is_modified());
    }

    #[test]
    fn test_audit_updated_now() {
        let mut audit = AuditFields::created("creator");
        assert_eq!(audit.created_by, "creator");
        assert_eq!(audit.updated_by, "creator");

        audit.updated("new_updater");
        assert_eq!(audit.created_by, "creator");
        assert_eq!(audit.updated_by, "new_updater");
    }

    #[test]
    fn test_audit_with_updated_builder() {
        let audit = AuditFields::created("creator").with_updated("updater");
        assert_eq!(audit.created_by, "creator");
        assert_eq!(audit.updated_by, "updater");
    }
}

