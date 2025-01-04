use std::collections::HashMap;
use std::error::Error;
use std::fmt::{format, Display};
use std::iter;

use anyhow::Result;

use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use serde_json_schema::StringOrStringArray;
use serde_json_schema::{BooleanOrSchema, Schema};

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

    first_letters + rest.as_str()
}

fn upper_camel_case(s: &str) -> String {
    s.split('-').map(capatalize).collect::<String>()
}

// TODO external schemas
// TODO validation>?
pub fn generate(root_schema: Schema) -> Result<String> {
    let type_mapping = TypeMapping::with_basic_types();
    let name = root_schema
        .title
        .clone()
        .or_else(|| {
            root_schema
                .get_id()
                .and_then(|url| get_first_resource_from_url(&url))
        })
        .map(|name| upper_camel_case(&name))
        .map(|name| Ident::new(&name, Span::call_site()))
        .ok_or(GeneratorError::NoNameForRootSchema)?;

    let mut fields: Vec<(&String, &Schema)> = root_schema.properties.inner_iter().collect();
    fields.sort_by(|a, b| String::cmp(a.0, b.0));
    let fields = fields
        .into_iter()
        .map(|(original_field_name, schema)| -> Result<TokenStream> {
            if schema.reference.is_some() {
                todo!()
            }

            let json_type = match schema.schema_type.clone().ok_or(
                GeneratorError::PropertyMissingTypeForField(original_field_name.to_owned()),
            )? {
                StringOrStringArray::String(value) => value,
                StringOrStringArray::Array(_) => todo!(),
            };

            let field_name = snake_case(original_field_name);

            let field_type = match json_type.as_str() {
                "array" => {
                    let items =
                        schema
                            .items
                            .clone()
                            .ok_or(GeneratorError::ArrayDoesNotHaveSchema(
                                original_field_name.to_owned(),
                            ))?;

                    let schema = match items {
                        BooleanOrSchema::Boolean(_) => Err(
                            GeneratorError::ArrayDoesNotHaveSchema(original_field_name.to_owned()),
                        )?,
                        BooleanOrSchema::InnerSchema(schema) => schema,
                    };

                    let json_type = match schema.schema_type.clone().ok_or(
                        GeneratorError::PropertyMissingTypeForField(original_field_name.to_owned()),
                    )? {
                        StringOrStringArray::String(value) => value,
                        StringOrStringArray::Array(_) => todo!(),
                    };

                    let inner_type = type_mapping.get(&json_type).ok_or(
                        GeneratorError::NoTypeMappingFoundForField(original_field_name.to_owned()),
                    )?;

                    let inner_type = Ident::new(inner_type, name.span());

                    quote! {Vec<#inner_type>}
                }
                "null" => todo!(),
                other => {
                    let field_type = type_mapping.get(other).ok_or(
                        GeneratorError::NoTypeMappingFoundForField(original_field_name.to_owned()),
                    )?;

                    let field_type = Ident::new(field_type, name.span());
                    quote! {#field_type}
                }
            };

            let field_type = root_schema
                .required
                .inner_iter()
                .find(|r| original_field_name == *r)
                .map(|_| quote! {#field_type})
                .unwrap_or(quote! {Option<#field_type>});

            let field_name = Ident::new(&field_name, name.span());

            let docs = schema.description.iter().map(|d| quote! {#[doc = #d]});

            let token_stream = quote! {
                #(#docs)*
                #field_name: #field_type
            };

            Ok(token_stream)
        })
        .collect::<Result<Vec<TokenStream>>>()?;

    let docs = iter::empty()
        .chain(root_schema.dollar_id.iter())
        .chain(root_schema.id.iter())
        .chain(root_schema.description.iter())
        .map(|s| s.to_owned())
        .map(|s| quote! {#[doc = #s]});

    let file_contents = quote! {
        #(#docs)*
        struct #name {
            #(#fields),*
        }
    };

    let syntax_tree = syn::parse2(file_contents).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);

    Ok(formatted)
}

#[derive(Debug)]
pub enum GeneratorError {
    NoNameForRootSchema,
    PropertyMissingTypeForField(String),
    NoTypeMappingFoundForField(String),
    ArrayDoesNotHaveSchema(String),
}

impl Error for GeneratorError {}

impl Display for GeneratorError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

// https://json-schema.org/understanding-json-schema/reference/type
pub struct TypeMapping {
    types: HashMap<String, String>,
}

impl TypeMapping {
    fn empty() -> Self {
        Self {
            types: HashMap::new(),
        }
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

pub trait InnerIterator<'a, R> {
    fn inner_iter(&'a self) -> impl Iterator<Item = R>;
}

impl<'a, K, V> InnerIterator<'a, (&'a K, &'a V)> for Option<HashMap<K, V>> {
    fn inner_iter(&'a self) -> impl Iterator<Item = (&'a K, &'a V)> {
        self.iter().flat_map(|map| map.iter())
    }
}

impl<'a, V> InnerIterator<'a, &'a V> for Option<Vec<V>> {
    fn inner_iter(&'a self) -> impl Iterator<Item = &'a V> {
        self.iter().flatten()
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
        let result = generate(schema).unwrap();

        let file_contents = quote! {
            ///https://example.com/person.schema.json
            struct Person {
                ///Age in years which must be equal to or greater than zero.
                age: Option<i64>,
                ///The person's first name.
                first_name: Option<String>,
                ///The person's last name.
                last_name: Option<String>,
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
        let result = generate(schema).unwrap();

        let file_contents = quote! {
            ///https://example.com/address.schema.json
            ///An address similar to http://microformats.org/wiki/h-card
            struct Address {
                country_name: String,
                extended_address: Option<String>,
                locality: String,
                post_office_box: Option<String>,
                postal_code: Option<String>,
                region: String,
                street_address: Option<String>,
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
        let result = generate(schema).unwrap();

        let file_contents = quote! {
            ///https://example.com/user-profile.schema.json
            ///A representation of a user profile
            struct UserProfile {
                age: Option<i64>,
                email: String,
                full_name: Option<String>,
                interests: Option<Vec<String>>,
                location: Option<String>,
                username: String,
            }
        };

        let syntax_tree = syn::parse2(file_contents).unwrap();
        let expected_result = prettyplease::unparse(&syntax_tree);

        assert_eq!(result, expected_result);
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

        let schema: Schema = serde_json::from_str(json_string).unwrap();
        let result = generate(schema).unwrap();

        let file_contents = quote! {
            ///https://example.com/blog-post.schema.json
            ///A representation of a blog post
            struct BlogPost {
                title: String,
                content: String,
                author: String,
                publishedDate: Option<String>,
                tags: Option<Vec<String>>,
            }
        };

        let syntax_tree = syn::parse2(file_contents).unwrap();
        let expected_result = prettyplease::unparse(&syntax_tree);

        assert_eq!(result, expected_result);
    }
}
