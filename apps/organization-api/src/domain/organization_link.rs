use crate::domain::OrganizationId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use tsid::{TSID, create_tsid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrganizationEntityType {
    Client,
    Producer,
}

impl OrganizationEntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Client => "CLIENT",
            Self::Producer => "PRODUCER",
        }
    }
}

impl fmt::Display for OrganizationEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for OrganizationEntityType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "CLIENT" => Ok(Self::Client),
            "PRODUCER" => Ok(Self::Producer),
            other => Err(format!("Unknown entity type: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationLink {
    pub id: TSID,
    pub organization_id: OrganizationId,
    pub entity_id: TSID,
    pub entity_type: OrganizationEntityType,
    pub linked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

impl OrganizationLink {
    pub fn new(
        organization_id: OrganizationId,
        entity_id: TSID,
        entity_type: OrganizationEntityType,
        created_by: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: create_tsid(),
            organization_id,
            entity_id,
            entity_type,
            linked_at: now,
            created_at: now,
            created_by: created_by.into(),
        }
    }

    pub fn with_id(mut self, id: TSID) -> Self {
        self.id = id;
        self
    }

    pub fn with_linked_at(mut self, linked_at: DateTime<Utc>) -> Self {
        self.linked_at = linked_at;
        self
    }

    pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn id(&self) -> TSID {
        self.id
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
    }

    pub fn entity_id(&self) -> TSID {
        self.entity_id
    }

    pub fn entity_type(&self) -> OrganizationEntityType {
        self.entity_type
    }

    pub fn linked_at(&self) -> DateTime<Utc> {
        self.linked_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn created_by(&self) -> &str {
        &self.created_by
    }
}
