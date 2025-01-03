use std::iter;

use serde_json_schema::{
    Schema, DEFINITIONS_PATH, DEPENDENT_SCHEMAS_PATH, PATTERN_PROPERTIES_PATH, PROPERTIES_PATH,
};

pub trait SchemaDiscoverable {
    fn discover(&self) -> SchemaDiscoverer;
}

impl SchemaDiscoverable for Schema {
    fn discover(&self) -> SchemaDiscoverer {
        SchemaDiscoverer::new(self)
    }
}

impl SchemaDiscoverable for &Schema {
    fn discover(&self) -> SchemaDiscoverer {
        SchemaDiscoverer::new(self)
    }
}

pub struct DiscoveredSchema<'a> {
    id: String,
    schema: &'a Schema,
}

impl<'a> DiscoveredSchema<'a> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn schema(&self) -> &'a Schema {
        self.schema
    }
}

pub struct SchemaDiscoverer<'a> {
    iter: std::vec::IntoIter<PathableSchema<'a>>,
    discovering: Option<std::vec::IntoIter<PathableSchema<'a>>>,
}

impl<'a> SchemaDiscoverer<'a> {
    fn new(schema: &'a Schema) -> Self {
        match schema.dollar_id.to_owned().or(schema.id.to_owned()) {
            Some(id) => {
                let pathable_schema = PathableSchema {
                    root_path: id.clone(),
                    path: id,
                    schema,
                };

                Self {
                    iter: pathable_schema.into_iter(),
                    discovering: None,
                }
            }
            None => Self {
                iter: Vec::new().into_iter(),
                discovering: None,
            },
        }
    }
}

impl<'a> From<PathableSchema<'a>> for DiscoveredSchema<'a> {
    fn from(value: PathableSchema<'a>) -> Self {
        DiscoveredSchema {
            id: value.path,
            schema: value.schema,
        }
    }
}

impl<'a> Iterator for SchemaDiscoverer<'a> {
    type Item = DiscoveredSchema<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = &mut self.discovering {
            if let Some(p) = iter.next() {
                return Some(p.into());
            }
        }

        let i = self.iter.next();
        self.discovering = i.clone().map(|pathable_schema| pathable_schema.into_iter());
        i.map(|p| p.into())
    }
}

#[derive(Clone)]
struct PathableSchema<'a> {
    root_path: String,
    path: String,
    schema: &'a Schema,
}

impl<'a> IntoIterator for PathableSchema<'a> {
    type Item = PathableSchema<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        if self.path.contains('#') {
            return Vec::new().into_iter();
        }

        let schemas: Vec<PathableSchema<'a>> = iter::empty()
            .chain(self.definitions())
            .chain(self.properties())
            .chain(self.dependent_schemas())
            .chain(self.pattern_properties())
            .flat_map(|p| match &p.schema.anchor {
                Some(anchor) => {
                    let original = p.clone();
                    let root_path = p.root_path;
                    let path = format!("{root_path}#{anchor}");

                    vec![
                        original,
                        PathableSchema {
                            root_path,
                            path,
                            schema: p.schema,
                        },
                    ]
                }
                None => Vec::new(),
            })
            .collect();

        schemas.into_iter()
    }
}

impl<'a> PathableSchema<'a> {
    fn definitions(&self) -> impl Iterator<Item = PathableSchema<'a>> + '_ {
        self.schema
            .definitions
            .iter()
            .flat_map(|map| map.iter())
            .map(|(key, schema)| {
                let root_path = match &schema.id {
                    Some(id) => id.to_string(),
                    None => self.root_path.clone(),
                };

                let path = format!("{root_path}/{DEFINITIONS_PATH}/{key}");
                PathableSchema {
                    root_path,
                    path,
                    schema,
                }
            })
    }

    fn properties(&self) -> impl Iterator<Item = PathableSchema<'a>> + '_ {
        self.schema
            .properties
            .iter()
            .flat_map(|map| map.iter())
            .map({
                let path = self.path.clone();
                move |(key, schema)| {
                    let path = format!("{path}/{PROPERTIES_PATH}/{key}");
                    PathableSchema {
                        root_path: self.root_path.clone(),
                        path,
                        schema,
                    }
                }
            })
    }

    fn dependent_schemas(&self) -> impl Iterator<Item = PathableSchema<'a>> + '_ {
        self.schema
            .dependent_schemas
            .iter()
            .flat_map(|map| map.iter())
            .map({
                let path = self.path.clone();
                move |(key, schema)| {
                    let path = format!("{path}/{DEPENDENT_SCHEMAS_PATH}/{key}");
                    PathableSchema {
                        root_path: self.root_path.clone(),
                        path,
                        schema,
                    }
                }
            })
    }

    fn pattern_properties(&self) -> impl Iterator<Item = PathableSchema<'a>> + '_ {
        self.schema
            .pattern_properties
            .iter()
            .flat_map(|map| map.iter())
            .map({
                let path = self.path.clone();
                move |(key, schema)| {
                    let path = format!("{path}/{PATTERN_PROPERTIES_PATH}/{key}");
                    PathableSchema {
                        root_path: self.root_path.clone(),
                        path,
                        schema,
                    }
                }
            })
    }
}
