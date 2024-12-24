use std::{collections::HashMap, iter};

use schema_discovery::SchemaDiscoverable;
use serde_json_schema::Schema;

pub enum SchemaRegistryIngestionError {
    NoInternalIdentifier,
    SchemaAlreadyExistsInRegistry,
}

pub enum SchemaDiscoveryError {
    EncounteredDuplicateSchema,
}

#[derive(Default)]
pub struct SchemaRegistry<'a> {
    discovered_schemas: HashMap<String, Schema>,
    schemas: HashMap<String, &'a Schema>,
    owned_schemas: HashMap<String, Schema>,
}

impl<'a> SchemaRegistry<'a> {
    pub fn add_internally_identified_schema(
        mut self,
        schema: &'a Schema,
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

        let mut schema = schema;
        schema.id = Some(external_id.clone());

        self.owned_schemas.insert(external_id, schema);
        Ok(self)
    }

    pub fn discover(mut self) -> Result<Self, SchemaDiscoveryError> {
        let result: Vec<(String, Schema)> = iter::empty()
            .chain(self.schemas.values().copied())
            .chain(self.owned_schemas.values())
            .flat_map(|schema| {
                schema
                    .discover()
                    .map(|d| (d.id().to_owned(), d.schema().clone()))
                    .collect::<Vec<(String, Schema)>>()
            })
            .collect();

        for (id, schema) in result {
            if self.schema_exists(&id) {
                return Err(SchemaDiscoveryError::EncounteredDuplicateSchema);
            }

            self.discovered_schemas.insert(id, schema);
        }

        Ok(self)
    }

    fn schema_exists(&self, id: &str) -> bool {
        self.schemas.contains_key(id)
            || self.owned_schemas.contains_key(id)
            || self.discovered_schemas.contains_key(id)
    }
}
