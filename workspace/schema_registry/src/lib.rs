use std::{collections::HashMap, error::Error, fmt::Display};

use schema_discovery::SchemaDiscoverable;
use serde_json_schema::Schema;

#[derive(Default)]
pub struct SchemaRegistry {
    discovered_schemas: HashMap<String, Schema>,
    schemas: HashMap<String, Schema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        SchemaRegistry::default()
    }

    pub fn add_internally_identified_schema(
        mut self,
        schema: Schema,
    ) -> Result<Self, SchemaRegistryIngestionError> {
        let id = schema
            .id
            .clone()
            .ok_or(SchemaRegistryIngestionError::NoInternalIdentifier)?;

        if self.schema_exists(&id) {
            return Err(SchemaRegistryIngestionError::SchemaAlreadyExistsInRegistry);
        }

        self.schemas.insert(id, schema);
        Ok(self)
    }

    pub fn add_externally_referenced_schema(
        mut self,
        external_id: String,
        schema: Schema,
    ) -> Result<Self, SchemaRegistryIngestionError> {
        if self.schema_exists(&external_id) {
            return Err(SchemaRegistryIngestionError::SchemaAlreadyExistsInRegistry);
        }

        self.schemas.insert(external_id, schema);
        Ok(self)
    }

    pub fn discover(mut self) -> Result<Self, SchemaRegistryDiscoveryError> {
        let iter = self.schemas.values().flat_map(|schema| {
            schema
                .discover()
                .map(|d| (d.id().to_string(), d.schema().clone()))
        });

        for (id, schema) in iter {
            if self.schema_exists(&id) {
                return Err(SchemaRegistryDiscoveryError::EncounteredDuplicateSchema);
            }

            self.discovered_schemas.insert(id, schema);
        }

        Ok(self)
    }

    pub fn get(&self, id: &str) -> Option<&Schema> {
        self.schemas.get(id)
    }

    fn schema_exists(&self, id: &str) -> bool {
        self.schemas.contains_key(id) || self.discovered_schemas.contains_key(id)
    }
}

#[derive(Debug)]
pub enum SchemaRegistryIngestionError {
    NoInternalIdentifier,
    SchemaAlreadyExistsInRegistry,
}

impl Error for SchemaRegistryIngestionError {}

impl Display for SchemaRegistryIngestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoInternalIdentifier => {
                write!(f, "Attempted to register an internally identified schema, but it didn't have an internal id.")
            }
            Self::SchemaAlreadyExistsInRegistry => {
                write!(
                    f,
                    "Attempted to register a schema, but there is an existing schema with that id."
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum SchemaRegistryDiscoveryError {
    EncounteredDuplicateSchema,
}

impl Error for SchemaRegistryDiscoveryError {}

impl Display for SchemaRegistryDiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncounteredDuplicateSchema => {
                write!(f, "Attempted to discover identifyable sub schemas from schemas in the repository, but while doing so encountered a duplicate.")
            }
        }
    }
}
