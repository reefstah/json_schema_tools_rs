use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::Display;
use std::iter;

use anyhow::Result;

use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use serde_json_schema::Schema;
use serde_json_schema::StringOrStringArray;

// TODO external schemas
// TODO rustify fieldnames
// TODO validation>?
pub fn generate(root_schema: Schema) -> Result<String> {
    let type_mapping = TypeMapping::with_basic_types();
    let name = root_schema
        .title
        .ok_or(GeneratorError::RootSchemaIsMissingATitle)?;
    let name = Ident::new(&name, Span::call_site());

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

            let first_letter = original_field_name
                .chars()
                .next()
                .unwrap_or_default()
                .to_lowercase()
                .next()
                .unwrap();

            let mut rest = original_field_name
                .chars()
                .skip(1)
                .flat_map(|letter| match letter.is_uppercase() {
                    true => vec!['_', letter.to_lowercase().next().unwrap()],
                    false => vec![letter],
                })
                .collect::<VecDeque<char>>();

            rest.push_front(first_letter);

            let field_name = rest.iter().collect::<String>();

            let field_type =
                match json_type.as_str() {
                    "array" => todo!(),
                    "null" => todo!(),
                    other => type_mapping.get(other).ok_or(
                        GeneratorError::NoTypeMappingFoundForField(field_name.to_owned()),
                    )?,
                };

            let field_type = Ident::new(field_type, name.span());
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
    RootSchemaIsMissingATitle,
    PropertyMissingTypeForField(String),
    NoTypeMappingFoundForField(String),
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
                "title": "Address",
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
}
