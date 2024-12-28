use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

use anyhow::Result;

use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use schema_registry::SchemaRegistry;
use serde_json_schema::Schema;
use serde_json_schema::StringOrStringArray;

// TODO external schemas
// TODO rustify fieldnames
pub fn generate(root_model_name: String, schema: Schema) -> Result<String> {
    // let out_dir = env::var_os("OUT_DIR").unwrap();
    // let file_name = format!("{}.rs", root_model_name);
    // let dest_path = Path::new(&out_dir).join(file_name);

    let _registry = SchemaRegistry::new()
        .add_internally_identified_schema(schema.clone())?
        .discover()?;

    let type_mapping = TypeMapping::with_basic_types();

    let name = Ident::new(&root_model_name, Span::call_site());

    let mut fields: Vec<(&String, &Schema)> = schema.properties.inner_iter().collect();
    fields.sort_by(|a, b| String::cmp(a.0, b.0));
    let fields = fields
        .into_iter()
        .map(|(field_name, schema)| -> Result<TokenStream> {
            if schema.reference.is_some() {
                todo!()
            }

            let json_type = match schema.schema_type.clone().ok_or(
                GeneratorError::PropertyMissingTypeForField(field_name.to_owned()),
            )? {
                StringOrStringArray::String(value) => value,
                StringOrStringArray::Array(_) => todo!(),
            };

            let field_type =
                match json_type.as_str() {
                    "array" => todo!(),
                    "null" => todo!(),
                    other => type_mapping.get(other).ok_or(
                        GeneratorError::NoTypeMappingFoundForField(field_name.to_owned()),
                    )?,
                };

            let field_name = Ident::new(field_name, name.span());
            let field_type = Ident::new(field_type, name.span());

            let token_stream = quote! {
                #field_name: #field_type
            };

            Ok(token_stream)
        })
        .collect::<Result<Vec<TokenStream>>>()?;

    let file_contents = quote! {
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
            .add("number", "u32")
            .add("integer", "u32")
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
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
                    }
                }
            }"##;

        let schema: Schema = serde_json::from_str(json_string).unwrap();
        let result = generate("UserProfile".to_owned(), schema).unwrap();

        let file_contents = r##"
            struct UserProfile {
                age: u32,
                email: String,
                fullName: String,
                location: String,
                username: String,
            }
        "##;

        let syntax_tree = syn::parse_str(file_contents).unwrap();
        let expected_result = prettyplease::unparse(&syntax_tree);

        assert_eq!(result, expected_result);
    }
}
