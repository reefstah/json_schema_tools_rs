use std::collections::HashMap;

use serde_json_schema::{
    BooleanOrSchema::{self, InnerSchema},
    Schema,
};

pub fn dereference(json_schema: Schema) -> Schema {
    let registry = populate_new_schema_registry(&json_schema);
    resolve(json_schema.clone(), &registry)
}

fn resolve(mut schema: Schema, registry: &HashMap<String, &Schema>) -> Schema {
    resolve_dictonary_schemas_if_available(&mut schema.properties, registry);
    resolve_dictonary_schemas_if_available(&mut schema.definitions, registry);
    resolve_dictonary_schemas_if_available(&mut schema.dependent_schemas, registry);

    resolve_inner_schema_if_available(&mut schema.items, registry);
    resolve_inner_schema_if_available(&mut schema.additional_items, registry);

    resolve_sub_schema_if_available(&mut schema.schema_if, registry);
    resolve_sub_schema_if_available(&mut schema.schema_then, registry);
    resolve_sub_schema_if_available(&mut schema.schema_else, registry);

    schema
}

fn resolve_dictonary_schemas_if_available(
    dictonary_schemas: &mut Option<HashMap<String, Schema>>,
    registry: &HashMap<String, &Schema>,
) {
    if let Some(ref mut schemas) = dictonary_schemas {
        for value in schemas.values_mut() {
            if let Some(reference) = &value.reference {
                let schema = *registry.get(reference).expect(reference);
                let schema = schema.clone();
                let resolved_schema = resolve(schema.clone(), registry);
                *value = resolved_schema;
            } else {
                *value = resolve(value.clone(), registry);
            }
        }
    }
}

fn resolve_inner_schema_if_available(
    inner_schema: &mut Option<BooleanOrSchema>,
    registry: &HashMap<String, &Schema>,
) {
    if let Some(InnerSchema(ref mut items)) = inner_schema {
        if let Some(reference) = &items.reference {
            let schema = *registry.get(reference).expect(reference);
            let schema = schema.clone();
            let resolved_schema = resolve(schema.clone(), registry);
            *items = Box::new(resolved_schema);
        };
    };
}

fn resolve_sub_schema_if_available(
    sub_schema: &mut Option<Box<Schema>>,
    registry: &HashMap<String, &Schema>,
) {
    if let Some(ref mut schema) = sub_schema {
        if let Some(reference) = &schema.reference {
            let registry_schema = *registry.get(reference).expect(reference);
            let registry_schema = registry_schema.clone();
            let resolved_registry_schema = resolve(registry_schema.clone(), registry);
            *schema = Box::new(resolved_registry_schema);
        }
    }
}

fn populate_new_schema_registry(json_schema: &Schema) -> HashMap<String, &Schema> {
    let registry = HashMap::new();
    populate_existing_schema_registry(registry, json_schema, String::from("#"))
}

fn populate_existing_schema_registry<'a>(
    mut registry: HashMap<String, &'a Schema>,
    json_schema: &'a Schema,
    current_path: String,
) -> HashMap<String, &'a Schema> {
    if let Some(definitions) = &json_schema.definitions {
        for (name, schema) in definitions {
            let current_path = current_path.clone();
            let path = format!("{current_path}/$defs/{name}");
            registry.insert(path, schema);
            registry = populate_existing_schema_registry(registry, schema, current_path);
        }
    }

    registry
}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_json_schema::Schema;

    /// https://json-schema.org/learn/miscellaneous-examples#arrays-of-things
    #[test]
    fn arrays_of_things_example() {
        let input_json_string = r##"{
                "$id": "https://example.com/arrays.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "Arrays of strings and objects",
                "title": "Arrays",
                "type": "object",
                "properties": {
                    "fruits": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    },
                    "vegetables": {
                        "type": "array",
                        "items": { "$ref": "#/$defs/veggie" }
                    }
                },
                "$defs": {
                    "veggie": {
                        "type": "object",
                        "required": [ "veggieName", "veggieLike" ],
                        "properties": {
                            "veggieName": {
                                "type": "string",
                                "description": "The name of the vegetable."
                            },
                            "veggieLike": {
                                "type": "boolean",
                                "description": "Do I like this vegetable?"
                            }
                        }
                    }
                }
            }"##;

        let output_json_string = r##"{
                "$id": "https://example.com/arrays.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "Arrays of strings and objects",
                "title": "Arrays",
                "type": "object",
                "properties": {
                    "fruits": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                    },
                    "vegetables": {
                        "type": "array",
                        "items": { 
                            "type": "object",
                            "required": [ "veggieName", "veggieLike" ],
                            "properties": {
                                "veggieName": {
                                    "type": "string",
                                    "description": "The name of the vegetable."
                                },
                                "veggieLike": {
                                    "type": "boolean",
                                    "description": "Do I like this vegetable?"
                                }
                            }
                        }
                    }
                },
                "$defs": {
                    "veggie": {
                        "type": "object",
                        "required": [ "veggieName", "veggieLike" ],
                        "properties": {
                            "veggieName": {
                                "type": "string",
                                "description": "The name of the vegetable."
                            },
                            "veggieLike": {
                                "type": "boolean",
                                "description": "Do I like this vegetable?"
                            }
                        }
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(input_json_string).unwrap();
        let dereferenced = dereference(deserialized);

        let actual_value: serde_json::Value = serde_json::to_value(dereferenced).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(output_json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }
}
