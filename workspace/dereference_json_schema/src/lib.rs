use std::{collections::HashMap, iter};

use serde_json_schema::{
    AnyType, BooleanOrIntegerOrNumber,
    BooleanOrSchema::{self, InnerSchema},
    IntegerOrNumber, PropertyNames, Schema, StringOrStringArray,
};

pub fn dereference(json_schema: Schema) -> Schema {
    let index = json_schema.to_index();

    let index = index.iter().fold(HashMap::new(), |mut map, (key, value)| {
        let schema = value.dereference(&index).unwrap();
        map.insert(key.clone(), schema);
        map
    });

    let ref_index = index
        .iter()
        .map(|(key, value)| (key.clone(), value))
        .collect();

    json_schema.dereference(&ref_index).unwrap()
}

trait IndexSchema {
    fn to_index(&self) -> HashMap<String, &Self> {
        let mut index = HashMap::new();
        self.index(&mut index, String::from("#"));
        index
    }

    fn index<'a>(&'a self, index: &mut HashMap<String, &'a Self>, path: String);
    fn index_anchors<'a>(&'a self, index: &mut HashMap<String, &'a Self>);
}

trait Derefence {
    fn dereference(self, index: &HashMap<String, &Schema>) -> Result<Schema, String>;
}

impl Derefence for &Schema {
    fn dereference(self, index: &HashMap<String, &Schema>) -> Result<Schema, String> {
        let mut schema = self
            .reference
            .as_ref()
            .filter(|reference| *reference != "#")
            .and_then(|reference| index.get(reference))
            .map(|referenced_schema| {
                referenced_schema
                    .dereference(index)
                    .and_then(|referenced_schema| self.merge(&referenced_schema))
            })
            .unwrap_or_else(|| Ok(self.clone()))?;

        schema.properties = schema
            .properties
            .iter()
            .flatten()
            .try_fold(HashMap::new(), |mut map, (key, schema)| {
                let deref = schema.dereference(index)?;
                map.insert(key.clone(), deref);
                Ok::<HashMap<std::string::String, Schema>, String>(map)
            })
            .map(|map| if map.is_empty() { None } else { Some(map) })?;

        schema.items = match schema.items {
            Some(BooleanOrSchema::InnerSchema(schema)) => {
                let deref = schema.dereference(index)?;
                Ok::<Option<BooleanOrSchema>, String>(Some(BooleanOrSchema::InnerSchema(Box::new(
                    deref,
                ))))
            }
            others => Ok(others),
        }?;

        Ok(schema)
    }
}

trait Merge {
    fn merge(&self, target: &Self) -> Result<Self, String>
    where
        Self: Sized;
}

impl Merge for Option<HashMap<String, Schema>> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self.iter())
            .chain(target.iter())
            .flatten()
            .try_fold(HashMap::new(), |mut map, entry| {
                if map.insert(entry.0.clone(), entry.1.clone()).is_none() {
                    Ok(map)
                } else {
                    Err(String::from("Clashing definition"))
                }
            })
            .map(|map| if map.is_empty() { None } else { Some(map) })
    }
}

impl Merge for Option<HashMap<String, Vec<String>>> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self.iter())
            .chain(target.iter())
            .flat_map(|map| map.iter())
            .try_fold(HashMap::new(), |mut map, entry| {
                if map.insert(entry.0.clone(), entry.1.clone()).is_none() {
                    Ok(map)
                } else {
                    Err(String::from("Clashing definition"))
                }
            })
            .map(|map| if map.is_empty() { None } else { Some(map) })
    }
}

impl Merge for Option<String> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| {
                if acc.is_some() {
                    Err(String::from("Clashing definition"))
                } else {
                    Ok(Some(value.clone()))
                }
            })
    }
}

impl Merge for Option<i64> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| {
                if acc.is_some() {
                    Err(String::from("Clashing definition"))
                } else {
                    Ok(Some(*value))
                }
            })
    }
}

impl Merge for Option<bool> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| {
                if acc.is_some() {
                    Err(String::from("Clashing definition"))
                } else {
                    Ok(Some(*value))
                }
            })
    }
}

impl Merge for Option<AnyType> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| {
                if acc.is_some() {
                    Err(String::from("Clashing definition"))
                } else {
                    Ok(Some(value.clone()))
                }
            })
    }
}

impl Merge for Option<BooleanOrIntegerOrNumber> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| {
                if acc.is_some() {
                    Err(String::from("Clashing definition"))
                } else {
                    Ok(Some(value.clone()))
                }
            })
    }
}

impl Merge for Option<PropertyNames> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| {
                if acc.is_some() {
                    Err(String::from("Clashing definition"))
                } else {
                    Ok(Some(value.clone()))
                }
            })
    }
}

impl Merge for Option<IntegerOrNumber> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| {
                if acc.is_some() {
                    Err(String::from("Clashing definition"))
                } else {
                    Ok(Some(value.clone()))
                }
            })
    }
}

impl Merge for Option<StringOrStringArray> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| match acc {
                Some(StringOrStringArray::String(_)) => Err(String::from("Clashing definition")),
                Some(StringOrStringArray::Array(prev)) => {
                    if let StringOrStringArray::Array(next) = value.clone() {
                        let items = iter::empty().chain(prev).chain(next).collect();
                        Ok(Some(StringOrStringArray::Array(items)))
                    } else {
                        Err(String::from("Clashing definition"))
                    }
                }
                None => Ok(Some(value.clone())),
            })
    }
}

impl Merge for Option<BooleanOrSchema> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, value| match acc {
                Some(BooleanOrSchema::Boolean(_)) => Err(String::from("Clashing definition")),
                Some(BooleanOrSchema::InnerSchema(prev)) => {
                    if let BooleanOrSchema::InnerSchema(next) = value.clone() {
                        Ok(Some(BooleanOrSchema::InnerSchema(Box::new(
                            prev.merge(&next)?,
                        ))))
                    } else {
                        Err(String::from("Clashing definition"))
                    }
                }
                None => Ok(Some(value.clone())),
            })
    }
}

impl Merge for Option<Vec<AnyType>> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, next| match acc {
                Some(prev) => Ok(Some(
                    iter::empty().chain(prev).chain(next.clone()).collect(),
                )),
                None => Ok(Some(next.clone())),
            })
    }
}

impl Merge for Option<Vec<Schema>> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc, next| match acc {
                Some(prev) => Ok(Some(
                    iter::empty().chain(prev).chain(next.clone()).collect(),
                )),
                None => Ok(Some(next.clone())),
            })
    }
}

impl Merge for Option<Box<Schema>> {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        iter::empty()
            .chain(self)
            .chain(target)
            .try_fold(None, |acc: Option<Box<Schema>>, next| match acc {
                Some(prev) => {
                    let schema = prev.merge(next)?;
                    Ok(Some(Box::new(schema)))
                }
                None => Ok(Some(next.clone())),
            })
    }
}

impl Merge for Schema {
    fn merge(&self, target: &Self) -> Result<Self, String> {
        Ok(Self {
            dollar_id: self.dollar_id.merge(&target.dollar_id)?,
            id: self.id.merge(&target.id)?,
            schema: self.schema.merge(&target.schema)?,
            title: self.title.merge(&target.title)?,
            schema_type: self.schema_type.merge(&target.schema_type)?,
            properties: self.properties.merge(&target.properties)?,
            description: self.description.merge(&target.description)?,
            minimum: self.minimum.merge(&target.minimum)?,
            min_length: self.min_length.merge(&target.min_length)?,
            max_length: self.max_length.merge(&target.max_length)?,
            definitions: self.definitions.merge(&target.definitions)?,
            required: self.required.merge(&target.required)?,
            items: self.items.merge(&target.items)?,
            additional_items: self.additional_items.merge(&target.items)?,
            reference: self.reference.clone(),
            schema_enum: self.schema_enum.merge(&target.schema_enum)?,
            pattern: self.pattern.merge(&target.pattern)?,
            dependent_required: self.dependent_required.merge(&target.dependent_required)?,
            dependent_schemas: self.dependent_schemas.merge(&target.dependent_schemas)?,
            schema_const: self.schema_const.merge(&target.schema_const)?,
            schema_if: self.schema_if.merge(&target.schema_if)?,
            schema_then: self.schema_then.merge(&target.schema_then)?,
            schema_else: self.schema_else.merge(&target.schema_else)?,
            format: self.format.merge(&target.format)?,
            one_of: self.one_of.merge(&target.one_of)?,
            all_of: self.all_of.merge(&target.all_of)?,
            any_of: self.any_of.merge(&target.any_of)?,
            not: self.not.merge(&target.not)?,
            anchor: self.anchor.merge(&target.anchor)?,
            maximum: self.maximum.merge(&target.maximum)?,
            multiple_of: self.multiple_of.merge(&target.multiple_of)?,
            exlusive_maximum: self.exlusive_maximum.merge(&target.exlusive_maximum)?,
            pattern_properties: self.pattern_properties.merge(&target.pattern_properties)?,
            additional_properties: self
                .additional_properties
                .merge(&target.additional_properties)?,
            unevaluated_properties: self
                .unevaluated_properties
                .merge(&target.unevaluated_properties)?,
            property_names: self.property_names.merge(&target.property_names)?,
            min_properties: self.min_properties.merge(&target.min_properties)?,
            max_properties: self.max_properties.merge(&target.max_properties)?,
            prefix_items: self.prefix_items.merge(&target.prefix_items)?,
            unevaluated_items: self.unevaluated_items.merge(&target.unevaluated_items)?,
            contains: self.contains.merge(&target.contains)?,
            min_contains: self.min_contains.merge(&target.min_contains)?,
            max_contains: self.max_contains.merge(&target.max_contains)?,
            min_items: self.min_items.merge(&target.min_items)?,
            max_items: self.max_items.merge(&target.max_items)?,
            unique_items: self.unique_items.merge(&target.unique_items)?,
            default: self.default.merge(&target.default)?,
            examples: self.examples.merge(&target.examples)?,
            deprecated: self.deprecated.merge(&target.deprecated)?,
            read_only: self.read_only.merge(&target.read_only)?,
            write_only: self.write_only.merge(&target.write_only)?,
            comment: self.comment.merge(&target.comment)?,
            content_encoding: self.content_encoding.merge(&target.content_encoding)?,
            content_media_type: self.content_media_type.merge(&target.content_media_type)?,
        })
    }
}

impl IndexSchema for Schema {
    fn index<'a>(&'a self, index: &mut HashMap<String, &'a Schema>, path: String) {
        self.index_anchors(index);

        if path == "#" || self.id.is_some() {
            self.definitions
                .iter()
                .flat_map(|definitions| definitions.iter())
                .for_each(|(key, value)| {
                    value.index(index, format!("{path}/$defs/{key}"));
                });
        }

        index.insert(path, self);
    }

    fn index_anchors<'a>(&'a self, index: &mut HashMap<String, &'a Schema>) {
        iter::empty()
            .chain(self.properties.iter())
            .chain(self.dependent_schemas.iter())
            .chain(self.pattern_properties.iter())
            .flat_map(|map| map.values())
            .for_each(|schema| {
                schema.index_anchors(index);
            });

        iter::empty()
            .chain(self.items.iter())
            .chain(self.additional_items.iter())
            .chain(self.additional_properties.iter())
            .chain(self.unevaluated_properties.iter())
            .chain(self.unevaluated_items.iter())
            .chain(self.contains.iter())
            .for_each(|item| {
                if let InnerSchema(schema) = item {
                    schema.index_anchors(index);
                }
            });

        iter::empty()
            .chain(self.schema_if.iter())
            .chain(self.schema_then.iter())
            .chain(self.schema_else.iter())
            .chain(self.not.iter())
            .for_each(|schema| {
                schema.index_anchors(index);
            });

        iter::empty()
            .chain(self.one_of.iter())
            .chain(self.all_of.iter())
            .chain(self.any_of.iter())
            .chain(self.prefix_items.iter())
            .flatten()
            .for_each(|schema| {
                schema.index_anchors(index);
            });

        if let Some(anchor) = &self.anchor {
            index.insert(format!("#{anchor}"), self);
        }
    }
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
                let mut resolved_schema = resolve(schema.clone(), registry);
                resolved_schema.anchor = None;
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
            let mut resolved_schema = resolve(schema.clone(), registry);
            resolved_schema.anchor = None;
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
            let mut resolved_registry_schema = resolve(registry_schema.clone(), registry);
            resolved_registry_schema.anchor = None;
            *schema = Box::new(resolved_registry_schema);
        }
    }
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
                            },
                            "$ref": "#/$defs/veggie"
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

    /// https://json-schema.org/learn/json-schema-examples#ecommerce-system
    #[test]
    fn ecommerce_system_example() {
        let input_json_string = r##"{
                "$id": "https://example.com/ecommerce.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "product": {
                        "$anchor": "ProductSchema",
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "price": { "type": "number", "minimum": 0 }
                        }
                    },
                    "order": {
                        "$anchor": "OrderSchema",
                        "type": "object",
                        "properties": {
                            "orderId": { "type": "string" },
                            "items": {
                                "type": "array",
                                "items": { "$ref": "#ProductSchema" }
                            }
                        }
                    }
                }
            }"##;

        let output_json_string = r##"{
                "$id": "https://example.com/ecommerce.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "product": {
                        "$anchor": "ProductSchema",
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "price": { "type": "number", "minimum": 0 }
                        }
                    },
                    "order": {
                        "$anchor": "OrderSchema",
                        "type": "object",
                        "properties": {
                            "orderId": { "type": "string" },
                            "items": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "name": { "type": "string" },
                                        "price": { "type": "number", "minimum": 0 }
                                    }
                                }
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
