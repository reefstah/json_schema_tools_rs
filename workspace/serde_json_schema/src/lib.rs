use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Schema {
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub dollar_id: Option<String>,

    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<StringOrStringArray>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,

    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i64>,

    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i64>,

    #[serde(rename = "$defs", skip_serializing_if = "Option::is_none")]
    pub definitions: Option<HashMap<String, Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<StringOrStringArray>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<BooleanOrSchema>,

    #[serde(rename = "additionalItems", skip_serializing_if = "Option::is_none")]
    pub additional_items: Option<BooleanOrSchema>,

    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub schema_enum: Option<Vec<AnyType>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    #[serde(rename = "dependentRequired", skip_serializing_if = "Option::is_none")]
    pub dependent_required: Option<HashMap<String, Vec<String>>>,

    #[serde(rename = "dependentSchemas", skip_serializing_if = "Option::is_none")]
    pub dependent_schemas: Option<HashMap<String, Schema>>,

    #[serde(rename = "const", skip_serializing_if = "Option::is_none")]
    pub schema_const: Option<AnyType>,

    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub schema_if: Option<Box<Schema>>,

    #[serde(rename = "then", skip_serializing_if = "Option::is_none")]
    pub schema_then: Option<Box<Schema>>,

    #[serde(rename = "else", skip_serializing_if = "Option::is_none")]
    pub schema_else: Option<Box<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(rename = "oneOf", skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Schema>>,

    #[serde(rename = "$anchor", skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,

    #[serde(rename = "multipleOf", skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<IntegerOrNumber>,

    #[serde(rename = "exclusiveMaximum", skip_serializing_if = "Option::is_none")]
    pub exlusive_maximum: Option<BooleanOrIntegerOrNumber>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AnyType {
    String(String),
    Array(Vec<AnyType>),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Null(Option<()>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum StringOrStringArray {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum IntegerOrNumber {
    Integer(i64),
    Number(f64),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum BooleanOrIntegerOrNumber {
    Integer(i64),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum BooleanOrSchema {
    Boolean(bool),
    InnerSchema(Box<Schema>),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// https://json-schema.org/understanding-json-schema/basics#hello-world!
    #[test]
    fn hello_world_example() {
        let empty_json_string = r"{}";
        let true_json_string = r"true";
        let false_json_string = r"false";

        let empty_deserialized: BooleanOrSchema = serde_json::from_str(empty_json_string).unwrap();
        let empty_actual_value: serde_json::Value =
            serde_json::to_value(empty_deserialized).unwrap();
        let empty_expected_value: serde_json::Value =
            serde_json::from_str(empty_json_string).unwrap();

        assert_eq!(empty_actual_value, empty_expected_value);

        let true_deserialized: BooleanOrSchema = serde_json::from_str(true_json_string).unwrap();
        let true_actual_value: serde_json::Value = serde_json::to_value(true_deserialized).unwrap();
        let true_expected_value: serde_json::Value =
            serde_json::from_str(true_json_string).unwrap();

        assert_eq!(true_actual_value, true_expected_value);

        let false_deserialized: BooleanOrSchema = serde_json::from_str(false_json_string).unwrap();
        let false_actual_value: serde_json::Value =
            serde_json::to_value(false_deserialized).unwrap();
        let false_expected_value: serde_json::Value =
            serde_json::from_str(false_json_string).unwrap();

        assert_eq!(false_actual_value, false_expected_value);
    }

    /// https://json-schema.org/understanding-json-schema/reference/type#type-specific-keywords
    #[test]
    fn type_specific_keywords_example() {
        let json_string = r#"{
                "type": ["number", "string"]
            }"#;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/understanding-json-schema/reference/string#length
    #[test]
    fn length_example() {
        let json_string = r#"{
                "type": "string",
                "minLength": 2,
                "maxLength": 3
            }"#;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/understanding-json-schema/reference/string#regexp
    #[test]
    fn regular_expression_example() {
        let json_string = r#"{
                "type": "string",
                "pattern": "^(\\([0-9]{3}\\))?[0-9]{3}-[0-9]{4}$"
            }"#;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/understanding-json-schema/reference/numeric#multiples
    #[test]
    fn multiples_example() {
        let json_string = r#"{
                "type": "number",
                "multipleOf" : 10
            }"#;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/understanding-json-schema/reference/numeric#range
    #[test]
    fn numeric_example() {
        let json_string = r#"{
                "type": "number",
                "minimum": 0,
                "exclusiveMaximum": 100
            }"#;

        let draft_4_json_string = r#"{
                "type": "number",
                "minimum": 0,
                "maximum": 100,
                "exclusiveMaximum": true
            }"#;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);

        let draft_4_deserialized: Schema = serde_json::from_str(draft_4_json_string).unwrap();

        let draft_4_actual_value: serde_json::Value =
            serde_json::to_value(draft_4_deserialized).unwrap();
        let draft_4_expected_value: serde_json::Value =
            serde_json::from_str(draft_4_json_string).unwrap();

        assert_eq!(draft_4_actual_value, draft_4_expected_value);
    }

    /// https://json-schema.org/learn/miscellaneous-examples#basic
    #[test]
    fn basic_example() {
        let json_string = r#"{
                "$id": "https://example.com/person.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "Person",
                "type": "object",
                "properties": {
                    "firstName": {
                    "type": "string",
                    "description": "The person's first name."
                    },
                    "lastName": {
                    "type": "string",
                    "description": "The person's last name."
                    },
                    "age": {
                    "description": "Age in years which must be equal to or greater than zero.",
                    "type": "integer",
                    "minimum": 0
                    }
                }
            }"#;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/miscellaneous-examples#arrays-of-things
    #[test]
    fn arrays_of_things_example() {
        let json_string = r##"{
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

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/miscellaneous-examples#enumerated-values
    #[test]
    fn enumerated_values_example() {
        let json_string = r##"{
                "$id": "https://example.com/enumerated-values.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "Enumerated Values",
                "type": "object",
                "properties": {
                    "data": {
                        "enum": [42, true, "hello", null, [1, 2, 3]]
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/miscellaneous-examples#regular-expression-pattern
    #[test]
    fn regex_expression_pattern_example() {
        let json_string = r##"{
                "$id": "https://example.com/regex-pattern.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "Regular Expression Pattern",
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "pattern": "^[A-Z]{3}-\\d{3}$"
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/miscellaneous-examples#complex-object-with-nested-properties
    #[test]
    fn complex_object_with_nested_properties_example() {
        let json_string = r##"{
                "$id": "https://example.com/complex-object.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "Complex Object",
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string"
                    },
                    "age": {
                        "type": "integer",
                        "minimum": 0
                    },
                    "address": {
                        "type": "object",
                        "properties": {
                            "street": {
                                "type": "string"
                            },
                            "city": {
                                "type": "string"
                            },
                            "state": {
                                "type": "string"
                            },
                            "postalCode": {
                                "type": "string",
                                "pattern": "\\d{5}"
                            }
                        },
                        "required": ["street", "city", "state", "postalCode"]
                    },
                    "hobbies": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    }
                },
                "required": ["name", "age"]
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/miscellaneous-examples#conditional-validation-with-dependentrequired
    #[test]
    fn conditional_validation_with_dependent_required_example() {
        let json_string = r##"{
                "$id": "https://example.com/conditional-validation-dependentRequired.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "Conditional Validation with dependentRequired",
                "type": "object",
                "properties": {
                    "foo": {
                    "type": "boolean"
                    },
                    "bar": {
                    "type": "string"
                    }
                },
                "dependentRequired": {
                    "foo": ["bar"]
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/miscellaneous-examples#conditional-validation-with-dependentschemas
    #[test]
    fn conditional_validation_with_dependent_schemas_example() {
        let json_string = r##"{
                "$id": "https://example.com/conditional-validation-dependentSchemas.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "Conditional Validation with dependentSchemas",
                "type": "object",
                "properties": {
                    "foo": {
                        "type": "boolean"
                    },
                    "propertiesCount": {
                        "type": "integer",
                        "minimum": 0
                    }
                },
                "dependentSchemas": {
                    "foo": {
                        "required": ["propertiesCount"],
                        "properties": {
                            "propertiesCount": {
                                "minimum": 7
                            }
                        }
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/miscellaneous-examples#conditional-validation-with-if-else
    #[test]
    fn conditional_validation_with_if_else_example() {
        let json_string = r##"{
                "$id": "https://example.com/conditional-validation-if-else.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "Conditional Validation with If-Else",
                "type": "object",
                "properties": {
                    "isMember": {
                        "type": "boolean"
                    },
                    "membershipNumber": {
                        "type": "string"
                    }
                },
                "required": ["isMember"],
                "if": {
                    "properties": {
                        "isMember": {
                            "const": true
                        }
                    }
                },
                "then": {
                    "properties": {
                        "membershipNumber": {
                            "type": "string",
                            "minLength": 10,
                            "maxLength": 10
                        }
                    }
                },
                "else": {
                    "properties": {
                        "membershipNumber": {
                            "type": "string",
                            "minLength": 15
                        }
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#address
    #[test]
    fn address_example() {
        let json_string = r##"{
                "$id": "https://example.com/address.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "An address similar to http://microformats.org/wiki/h-card",
                "type": "object",
                "properties": {
                        "postOfficeBox": {
                        "type": "string"
                    },
                    "extendedAddress": {
                        "type": "string"
                    },
                    "streetAddress": {
                        "type": "string"
                    },
                    "locality": {
                        "type": "string"
                    },
                    "region": {
                        "type": "string"
                    },
                    "postalCode": {
                        "type": "string"
                    },
                    "countryName": {
                        "type": "string"
                    }
                },
                "required": [ "locality", "region", "countryName" ],
                "dependentRequired": {
                    "postOfficeBox": [ "streetAddress" ],
                    "extendedAddress": [ "streetAddress" ]
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#blog-post
    #[test]
    fn blog_post_example() {
        let json_string = r##"{
                "$id": "https://example.com/blog-post.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "A representation of a blog post",
                "type": "object",
                "required": ["title", "content", "author"],
                "properties": {
                    "title": {
                        "type": "string"
                    },
                    "content": {
                        "type": "string"
                    },
                    "publishedDate": {
                        "type": "string",
                        "format": "date-time"
                    },
                    "author": {
                        "$ref": "https://example.com/user-profile.schema.json"
                    },
                    "tags": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#calendar
    #[test]
    fn calendar_example() {
        let json_string = r##"{
                "$id": "https://example.com/calendar.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "A representation of an event",
                "type": "object",
                "required": [ "dtstart", "summary" ],
                "properties": {
                    "startDate": {
                        "type": "string",
                        "description": "Event starting time"
                    },
                    "endDate": {
                        "type": "string",
                        "description": "Event ending time"
                    },
                    "summary": {
                        "type": "string"
                    },
                    "location": {
                        "type": "string"
                    },
                    "url": {
                        "type": "string"
                    },
                    "duration": {
                        "type": "string",
                        "description": "Event duration"
                    },
                    "recurrenceDate": {
                        "type": "string",
                        "description": "Recurrence date"
                    },
                    "recurrenceDule": {
                        "type": "string",
                        "description": "Recurrence rule"
                    },
                    "category": {
                        "type": "string"
                    },
                    "description": {
                        "type": "string"
                    },
                    "geo": {
                        "$ref": "https://example.com/geographical-location.schema.json"
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#device-type
    #[test]
    fn device_type() {
        let root_json_string = r##"{
                "$id": "https://example.com/device.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {
                    "deviceType": {
                        "type": "string"
                    }
                },
                "required": ["deviceType"],
                "oneOf": [
                    {
                        "properties": {
                            "deviceType": { "const": "smartphone" }
                        },
                        "$ref": "https://example.com/smartphone.schema.json"
                    },
                    {
                        "properties": {
                            "deviceType": { "const": "laptop" }
                        },
                        "$ref": "https://example.com/laptop.schema.json"
                    }
                ]
            }"##;

        let smart_phone_json_string = r##"{
                "$id": "https://example.com/smartphone.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {
                    "brand": {
                        "type": "string"
                    },
                    "model": {
                        "type": "string"
                    },
                    "screenSize": {
                        "type": "number"
                    }
                },
                "required": ["brand", "model", "screenSize"]
            }"##;

        let laptop_json_string = r##"{
                "$id": "https://example.com/laptop.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {
                    "brand": {
                        "type": "string"
                    },
                    "model": {
                        "type": "string"
                    },
                    "processor": {
                        "type": "string"
                    },
                    "ramSize": {
                        "type": "number"
                    }
                },
                "required": ["brand", "model", "processor", "ramSize"]
            }"##;

        let root_deserialized: Schema = serde_json::from_str(root_json_string).unwrap();
        let root_actual_value: serde_json::Value = serde_json::to_value(root_deserialized).unwrap();
        let root_expected_value: serde_json::Value =
            serde_json::from_str(root_json_string).unwrap();

        assert_eq!(root_actual_value, root_expected_value);

        let smart_phone_deserialized: Schema =
            serde_json::from_str(smart_phone_json_string).unwrap();
        let smart_phone_actual_value: serde_json::Value =
            serde_json::to_value(smart_phone_deserialized).unwrap();
        let smart_phone_expected_value: serde_json::Value =
            serde_json::from_str(smart_phone_json_string).unwrap();

        assert_eq!(smart_phone_actual_value, smart_phone_expected_value);

        let laptop_deserialized: Schema = serde_json::from_str(laptop_json_string).unwrap();
        let laptop_actual_value: serde_json::Value =
            serde_json::to_value(laptop_deserialized).unwrap();
        let laptop_expected_value: serde_json::Value =
            serde_json::from_str(laptop_json_string).unwrap();

        assert_eq!(laptop_actual_value, laptop_expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#ecommerce-system
    #[test]
    fn ecommerce_system_example() {
        let json_string = r##"{
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

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#geographical-location
    #[test]
    fn geographical_location_example() {
        let json_string = r##"{
                "$id": "https://example.com/geographical-location.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "Longitude and Latitude Values",
                "description": "A geographical coordinate.",
                "required": [ "latitude", "longitude" ],
                "type": "object",
                "properties": {
                    "latitude": {
                        "type": "number",
                        "minimum": -90,
                        "maximum": 90
                    },
                        "longitude": {
                        "type": "number",
                        "minimum": -180,
                        "maximum": 180
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#health-record
    #[test]
    fn health_record_example() {
        let json_string = r##"{
                "$id": "https://example.com/health-record.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "Schema for representing a health record",
                "type": "object",
                "required": ["patientName", "dateOfBirth", "bloodType"],
                "properties": {
                    "patientName": {
                        "type": "string"
                    },
                    "dateOfBirth": {
                        "type": "string",
                        "format": "date"
                    },
                    "bloodType": {
                        "type": "string"
                    },
                    "allergies": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    },
                    "conditions": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    },
                    "medications": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    },
                    "emergencyContact": {
                        "$ref": "https://example.com/user-profile.schema.json"
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#job-posting
    #[test]
    fn job_posting_example() {
        let json_string = r##"{
                "$id": "https://example.com/job-posting.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "A representation of a job posting",
                "type": "object",
                "required": ["title", "company", "location", "description"],
                "properties": {
                    "title": {
                        "type": "string"
                    },
                    "company": {
                        "type": "string"
                    },
                    "location": {
                        "type": "string"
                    },
                    "description": {
                        "type": "string"
                    },
                    "employmentType": {
                        "type": "string"
                    },
                    "salary": {
                        "type": "number",
                        "minimum": 0
                    },
                    "applicationDeadline": {
                        "type": "string",
                        "format": "date"
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#movie
    #[test]
    fn movie_example() {
        let json_string = r##"{
                "$id": "https://example.com/movie.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "A representation of a movie",
                "type": "object",
                "required": ["title", "director", "releaseDate"],
                "properties": {
                    "title": {
                        "type": "string"
                    },
                    "director": {
                        "type": "string"
                    },
                    "releaseDate": {
                        "type": "string",
                        "format": "date"
                    },
                    "genre": {
                        "type": "string",
                        "enum": ["Action", "Comedy", "Drama", "Science Fiction"]
                    },
                    "duration": {
                        "type": "string"
                    },
                    "cast": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "additionalItems": false
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }

    /// https://json-schema.org/learn/json-schema-examples#user-profile
    #[test]
    fn user_profile_example() {
        let json_string = r##"{
                "$id": "https://example.com/user-profile.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "A representation of a user profile",
                "type": "object",
                "required": ["username", "email"],
                "properties": {
                    "username": {
                        "type": "string"
                    },
                    "email": {
                        "type": "string",
                        "format": "email"
                    },
                    "fullName": {
                        "type": "string"
                    },
                    "age": {
                        "type": "integer",
                        "minimum": 0
                    },
                    "location": {
                        "type": "string"
                    },
                    "interests": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        }
                    }
                }
            }"##;

        let deserialized: Schema = serde_json::from_str(json_string).unwrap();

        let actual_value: serde_json::Value = serde_json::to_value(deserialized).unwrap();
        let expected_value: serde_json::Value = serde_json::from_str(json_string).unwrap();

        assert_eq!(actual_value, expected_value);
    }
}