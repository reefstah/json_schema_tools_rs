use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::iter;

use anyhow::Result;

use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use schema_discovery::{SchemaDiscoverable, SchemaDiscoverer};
use schema_registry::SchemaRegistry;
use serde_json_schema::StringOrStringArray;
use serde_json_schema::{BooleanOrSchema, Schema};

// TODO validation>?
// TODO schema wrapper with name or derived name
// TODO generate from references instead of owned objects
// TODO attempt to order generated classes from parent to children
// TODO replace reliance on titles for nested objects with property names instead
// TODO allow for customisable module path

fn capatalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first_letter) => first_letter.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn get_first_resource_from_url(url: &str) -> Option<String> {
    url.split('/').last().map(|s| s.to_owned()).map(|url_end| {
        url_end
            .chars()
            .take_while(|c| *c != '.')
            .collect::<String>()
    })
}

fn snake_case(s: &str) -> String {
    let mut chars = s.chars();

    let first_letters = chars
        .next()
        .unwrap_or_default()
        .to_lowercase()
        .collect::<String>();

    let rest = chars
        .map(|letter| match letter.is_uppercase() {
            true => format!("_{}", letter.to_lowercase().collect::<String>()),
            false => letter.to_string(),
        })
        .collect::<String>();

    (first_letters + rest.as_str()).replace('-', "_")
}

fn upper_camel_case(s: &str) -> String {
    s.split('-').map(capatalize).collect::<String>()
}

fn struct_name(schema: &Schema) -> Option<String> {
    schema
        .title
        .clone()
        .or_else(|| {
            schema
                .get_id()
                .and_then(|url| get_first_resource_from_url(&url))
        })
        .map(|name| upper_camel_case(&name))
}

struct ModuleGenerator<'a> {
    root_schema: Option<Schema>,
    discover: SchemaDiscoverer<'a>,
    type_mapping: &'a TypeMapping,
    registry: &'a SchemaRegistry,
}

impl<'a> Iterator for ModuleGenerator<'a> {
    type Item = Result<TokenStream>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.discover.next() {
            Some(discovered_schema) => {
                let root_schema_id = discovered_schema.root_schema_id().to_owned();

                let root_schema = self
                    .root_schema
                    .as_ref()
                    .filter(|schema| {
                        // FIXME unwrap here
                        schema.get_id().is_some() && schema.get_id().unwrap() == root_schema_id
                    })
                    .take()
                    .map(|schema| (schema.to_owned(), schema.get_id().unwrap()));

                let same_root_iter = self
                    .discover
                    .by_ref()
                    .take_while(|next| root_schema_id == next.root_schema_id());

                let module = iter::once(discovered_schema)
                    .chain(same_root_iter)
                    .filter(|d| match &d.schema().schema_type {
                        Some(StringOrStringArray::String(value)) => *value == "object",
                        _ => false,
                    })
                    .map(|discovered_schema| {
                        (
                            discovered_schema.schema().to_owned(),
                            discovered_schema.root_schema_id().to_owned(),
                        )
                    })
                    .chain(root_schema)
                    .map(|(schema, root_schema_id)| {
                        to_struct(schema, root_schema_id, self.type_mapping, self.registry)
                    })
                    .collect::<Result<TokenStream>>()
                    .map(|structs| {
                        quote! {
                            use serde::{Serialize, Deserialize};

                            #structs
                        }
                    });

                Some(module)
            }
            None => None,
        }
    }
}

fn to_struct(
    schema: Schema,
    root_schema_id: String,
    type_mapping: &TypeMapping,
    registry: &SchemaRegistry,
) -> Result<TokenStream> {
    let name = struct_name(&schema)
        .map(|name| Ident::new(&name, Span::call_site()))
        .ok_or(GeneratorError::NoNameForRootSchema)?;

    let schema_id = schema.get_id();
    let docs = iter::empty()
        .chain(schema_id)
        .chain(schema.description.clone())
        .map(|s| s.to_owned())
        .map(|s| quote! {#[doc = #s]});

    let mut fields = Vec::new();
    let generator =
        FieldGenerator::new(schema, name.span(), root_schema_id, type_mapping, registry);

    for result in generator {
        let field = result?;
        fields.push(field);
    }

    Ok(quote! {
        #(#docs)*
        #[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct #name {
            #(#fields),*
        }
    })
}

struct FieldGenerator<'a> {
    struct_span: Span,
    properties: Vec<(String, Schema)>,
    schema: Schema,
    root_schema_id: String,
    type_mapping: &'a TypeMapping,
    registry: &'a SchemaRegistry,
}

impl<'a> Iterator for FieldGenerator<'a> {
    type Item = Result<TokenStream>;

    fn next(&mut self) -> Option<Self::Item> {
        self.properties
            .pop()
            .map(|(property_name, schema)| self.next_field(property_name, schema))
    }
}

impl<'a> FieldGenerator<'a> {
    fn new(
        mut schema: Schema,
        struct_span: Span,
        root_schema_id: String,
        type_mapping: &'a TypeMapping,
        registry: &'a SchemaRegistry,
    ) -> Self {
        let mut properties = schema
            .properties
            .take()
            .into_iter()
            .flatten()
            .collect::<Vec<(String, Schema)>>();

        properties.sort_by(|a, b| String::cmp(&b.0, &a.0));

        FieldGenerator {
            struct_span,
            properties,
            schema,
            root_schema_id,
            type_mapping,
            registry,
        }
    }

    fn next_field(&mut self, property_name: String, schema: Schema) -> Result<TokenStream> {
        let (schema, root_schema_id) = match schema.reference {
            Some(reference) => {
                let reference_root = reference
                    .split_once('#')
                    .map(|r| r.0.to_owned())
                    .unwrap_or(reference.clone());

                let schema = self
                    .registry
                    .get(&reference)
                    .ok_or(GeneratorError::UnresolvableReference(reference))?
                    .to_owned();

                (schema, reference_root)
            }
            None => (schema, self.root_schema_id.clone()),
        };

        let json_type =
            match schema
                .schema_type
                .clone()
                .ok_or(GeneratorError::PropertyMissingTypeForField(
                    property_name.to_owned(),
                ))? {
                StringOrStringArray::String(value) => value,
                StringOrStringArray::Array(_) => todo!(),
            };

        let field_name = snake_case(&property_name);

        let field_type = match json_type.as_str() {
            "object" => {
                let field_type = struct_name(&schema)
                    .ok_or(GeneratorError::NoNameForTypeofField(property_name.clone()))?;

                let module_name = get_first_resource_from_url(&root_schema_id)
                    .map(|resource_name| snake_case(&resource_name))
                    .ok_or(GeneratorError::NoNameForRootSchema)?;

                let field_type = Ident::new(&field_type, self.struct_span);
                let module_name = Ident::new(&module_name, self.struct_span);

                quote! {crate::#module_name::#field_type}
            }
            "array" => {
                let items = schema
                    .items
                    .clone()
                    .ok_or(GeneratorError::ArrayDoesNotHaveSchema(
                        property_name.to_owned(),
                    ))?;

                let schema = match items {
                    BooleanOrSchema::Boolean(_) => Err(GeneratorError::ArrayDoesNotHaveSchema(
                        property_name.to_owned(),
                    ))?,
                    BooleanOrSchema::InnerSchema(schema) => schema,
                };

                let json_type = match schema.schema_type.clone().ok_or(
                    GeneratorError::PropertyMissingTypeForField(property_name.to_owned()),
                )? {
                    StringOrStringArray::String(value) => value,
                    StringOrStringArray::Array(_) => todo!(),
                };

                // FIXME implement iterator for recursive behaviour
                if json_type == "object" {
                    let field_type = struct_name(&schema)
                        .ok_or(GeneratorError::NoNameForTypeofField(property_name.clone()))?;

                    let module_name = get_first_resource_from_url(&root_schema_id)
                        .map(|resource_name| snake_case(&resource_name))
                        .ok_or(GeneratorError::NoNameForRootSchema)?;

                    let field_type = Ident::new(&field_type, self.struct_span);
                    let module_name = Ident::new(&module_name, self.struct_span);

                    quote! {Vec<crate::#module_name::#field_type>}
                } else {
                    let inner_type = self.type_mapping.get(&json_type).ok_or(
                        GeneratorError::NoTypeMappingFoundForField(property_name.to_owned()),
                    )?;

                    let inner_type = Ident::new(inner_type, self.struct_span);

                    quote! {Vec<#inner_type>}
                }
            }
            "null" => todo!(),
            other => {
                let field_type = self.type_mapping.get(other).ok_or(
                    GeneratorError::NoTypeMappingFoundForField(property_name.to_owned()),
                )?;

                let field_type = Ident::new(field_type, self.struct_span);
                quote! {#field_type}
            }
        };

        let field_type = self
            .schema
            .required
            .iter()
            .flatten()
            .find(|r| property_name == **r)
            .map(|_| quote! {#field_type})
            .unwrap_or(quote! {Option<#field_type>});

        let field_name = Ident::new(&field_name, self.struct_span);
        let docs = schema.description.iter().map(|d| quote! {#[doc = #d]});

        Ok(quote! {
            #(#docs)*
            #[serde(rename = #property_name)]
            pub #field_name: #field_type
        })
    }
}

#[derive(Default)]
pub struct Generator {
    queued_schemas: Vec<Schema>,
    type_mapping: TypeMapping,
    registry: SchemaRegistry,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            queued_schemas: Vec::new(),
            type_mapping: TypeMapping::with_basic_types(),
            registry: SchemaRegistry::default(),
        }
    }

    pub fn schema_registry(mut self, registry: SchemaRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn single(mut self, schema: Schema) -> Self {
        self.queued_schemas.push(schema);
        self
    }

    pub fn many(mut self, schemas: &mut Vec<Schema>) -> Self {
        self.queued_schemas.append(schemas);
        self
    }

    pub fn generate(&self, schema: Schema) -> Result<String> {
        let mut generator = ModuleGenerator {
            root_schema: Some(schema.clone()),
            discover: schema.discover(),
            type_mapping: &self.type_mapping,
            registry: &self.registry,
        };

        let module = generator.next().ok_or(GeneratorError::NoSchemasFound)??;
        let syntax_tree = syn::parse2(module).unwrap();
        let formatted = prettyplease::unparse(&syntax_tree);
        Ok(formatted)
    }
}

#[derive(Debug)]
pub enum GeneratorError {
    NoSchemasFound,
    NoNameForRootSchema,
    NoNameForTypeofField(String),
    PropertyMissingTypeForField(String),
    NoTypeMappingFoundForField(String),
    ArrayDoesNotHaveSchema(String),
    UnresolvableReference(String),
}

impl Error for GeneratorError {}

impl Display for GeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSchemasFound => write!(f, "No schemas found"),
            Self::NoNameForRootSchema => write!(f, "No name for root schema"),
            Self::NoNameForTypeofField(field) => write!(f, "No name for type of field {}", field),
            Self::PropertyMissingTypeForField(field) => {
                write!(f, "Property missing type for field {}", field)
            }
            Self::NoTypeMappingFoundForField(field) => {
                write!(f, "No type mapping found for field {}", field)
            }
            Self::ArrayDoesNotHaveSchema(field) => {
                write!(f, "Array does not have schema {}", field)
            }
            Self::UnresolvableReference(field) => {
                write!(f, "Reference {} is not resolvable", field)
            }
        }
    }
}

// https://json-schema.org/understanding-json-schema/reference/type
#[derive(Default)]
pub struct TypeMapping {
    types: HashMap<String, String>,
}

impl TypeMapping {
    fn empty() -> Self {
        TypeMapping::default()
    }

    pub fn with_basic_types() -> Self {
        Self::empty()
            .add("string", "String")
            .add("number", "f64")
            .add("integer", "i64")
            .add("boolean", "bool")
    }

    fn add(mut self, json_type: &str, language_type: &str) -> Self {
        self.types
            .insert(json_type.to_owned(), language_type.to_owned());
        self
    }

    pub fn get(&self, json_type: &str) -> Option<&String> {
        self.types.get(json_type)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// https://json-schema.org/learn/miscellaneous-examples#basic
    #[test]
    fn basic_example() {
        let json_string = r##"{
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
            }"##;

        let schema: Schema = serde_json::from_str(json_string).unwrap();
        let result = Generator::new().generate(schema).unwrap();

        let file_contents = quote! {
            use serde::{Serialize, Deserialize};

            ///https://example.com/person.schema.json
            #[derive(Debug, Serialize, Deserialize)]
            pub struct Person {
                ///Age in years which must be equal to or greater than zero.
                #[serde(rename = "age")]
                pub age: Option<i64>,
                ///The person's first name.
                #[serde(rename = "firstName")]
                pub first_name: Option<String>,
                ///The person's last name.
                #[serde(rename = "lastName")]
                pub last_name: Option<String>,
            }
        };

        let syntax_tree = syn::parse2(file_contents).unwrap();
        let expected_result = prettyplease::unparse(&syntax_tree);

        assert_eq!(result, expected_result);
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

        let schema: Schema = serde_json::from_str(json_string).unwrap();
        let result = Generator::new().generate(schema).unwrap();

        let file_contents = quote! {
            ///https://example.com/address.schema.json
            ///An address similar to http://microformats.org/wiki/h-card
            pub struct Address {
                #[serde(rename = "countryName")]
                pub country_name: String,
                #[serde(rename = "extendedAddress")]
                pub extended_address: Option<String>,
                #[serde(rename = "locality")]
                pub locality: String,
                #[serde(rename = "postOfficeBox")]
                pub post_office_box: Option<String>,
                #[serde(rename = "postalCode")]
                pub postal_code: Option<String>,
                #[serde(rename = "region")]
                pub region: String,
                #[serde(rename = "streetAddress")]
                pub street_address: Option<String>,
            }
        };

        let syntax_tree = syn::parse2(file_contents).unwrap();
        let expected_result = prettyplease::unparse(&syntax_tree);

        assert_eq!(result, expected_result);
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

        let schema: Schema = serde_json::from_str(json_string).unwrap();
        let result = Generator::new().generate(schema).unwrap();

        let file_contents = quote! {
            ///https://example.com/user-profile.schema.json
            ///A representation of a user profile
            pub struct UserProfile {
                #[serde(rename = "age")]
                pub age: Option<i64>,
                #[serde(rename = "email")]
                pub email: String,
                #[serde(rename = "fullName")]
                pub full_name: Option<String>,
                #[serde(rename = "interests")]
                pub interests: Option<Vec<String>>,
                #[serde(rename = "location")]
                pub location: Option<String>,
                #[serde(rename = "username")]
                pub username: String,
            }
        };

        let syntax_tree = syn::parse2(file_contents).unwrap();
        let expected_result = prettyplease::unparse(&syntax_tree);

        assert_eq!(result, expected_result);
    }

    /// https://json-schema.org/learn/json-schema-examples#blog-post
    #[test]
    fn blog_post_example() {
        let user_profile_json_string = r##"{
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

        let user_profile_schema: Schema = serde_json::from_str(user_profile_json_string).unwrap();

        let registry = SchemaRegistry::new()
            .add_internally_identified_schema(user_profile_schema)
            .unwrap()
            .discover()
            .unwrap();

        let blog_post_json_string = r##"{
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

        let blog_post_schema: Schema = serde_json::from_str(blog_post_json_string).unwrap();

        let result = Generator::new()
            .schema_registry(registry)
            .generate(blog_post_schema)
            .unwrap();

        let file_contents = quote! {
            ///https://example.com/blog-post.schema.json
            ///A representation of a blog post
            pub struct BlogPost {
                ///A representation of a user profile
                #[serde(rename = "author")]
                pub author: crate::user_profile::UserProfile,
                #[serde(rename = "content")]
                pub content: String,
                #[serde(rename = "publishedDate")]
                pub published_date: Option<String>,
                #[serde(rename = "tags")]
                pub tags: Option<Vec<String>>,
                #[serde(rename = "title")]
                pub title: String,
            }
        };

        let syntax_tree = syn::parse2(file_contents).unwrap();
        let expected_result = prettyplease::unparse(&syntax_tree);

        assert_eq!(result, expected_result);
    }
}

/// https://json-schema.org/learn/miscellaneous-examples#arrays-of-things
#[test]
fn arrays_of_things_modified_example() {
    let json_string = r##"{
                "$id": "https://example.com/arrays.schema.json",
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "description": "Arrays of strings and objects",
                "title": "Arrays",
                "type": "object",
                "properties": {
                    "bowl": {
                        "title": "Bowl",
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
                                    "title": "Veggie",
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
                        }
                    }
                }
            }"##;

    let schema: Schema = serde_json::from_str(json_string).unwrap();
    let result = Generator::new().generate(schema).unwrap();

    let file_contents = quote! {
        pub struct Bowl {
            #[serde(rename = "fruits")]
            pub fruits: Option<Vec<String>>,
            #[serde(rename = "vegetables")]
            pub vegetables: Option<Vec<crate::arrays::Veggie>>,
        }

        pub struct Veggie {
            ///Do I like this vegetable?
            #[serde(rename = "veggieLike")]
            pub veggie_like: bool,
            ///The name of the vegetable.
            #[serde(rename = "veggieName")]
            pub veggie_name: String,
        }

        ///https://example.com/arrays.schema.json
        ///Arrays of strings and objects
        pub struct Arrays {
            #[serde(rename = "bowl")]
            pub bowl: Option<crate::arrays::Bowl>
        }
    };

    let syntax_tree = syn::parse2(file_contents).unwrap();
    let expected_result = prettyplease::unparse(&syntax_tree);

    assert_eq!(result, expected_result);
}
