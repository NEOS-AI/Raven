use crate::error::{Result, SearchEngineError};
use crate::types::{FieldType, FieldValue, SchemaDefinition};
use std::collections::HashMap;
use tantivy::schema::{
    DateOptions, Field, INDEXED, NumericOptions, STORED, STRING, Schema, SchemaBuilder, TEXT,
    TextFieldIndexing, TextOptions, Value,
};

/// Schema manager for handling Tantivy schemas
#[derive(Debug, Clone)]
pub struct SchemaManager {
    schema_def: SchemaDefinition,
    tantivy_schema: Schema,
    field_map: HashMap<String, Field>,
}

impl SchemaManager {
    /// Create a new schema manager from schema definition
    pub fn new(schema_def: SchemaDefinition) -> Result<Self> {
        let (tantivy_schema, field_map) = Self::build_tantivy_schema(&schema_def)?;

        Ok(Self {
            schema_def,
            tantivy_schema,
            field_map,
        })
    }

    /// Build Tantivy schema from our schema definition
    fn build_tantivy_schema(
        schema_def: &SchemaDefinition,
    ) -> Result<(Schema, HashMap<String, Field>)> {
        let mut schema_builder = SchemaBuilder::new();
        let mut field_map = HashMap::new();

        // Add ID field (always present)
        let id_field = schema_builder.add_text_field("_id", TEXT | STORED);
        field_map.insert("_id".to_string(), id_field);

        // Add user-defined fields
        for (field_name, field_type) in &schema_def.fields {
            let field = match field_type {
                FieldType::Text {
                    stored,
                    indexed,
                    tokenizer,
                } => {
                    let mut options = TextOptions::default();

                    if *stored {
                        options = options.set_stored();
                    }

                    if *indexed {
                        // Handle keyword tokenizer separately
                        if tokenizer == "keyword" {
                            // For exact matching, use STRING field
                            if *stored {
                                let field =
                                    schema_builder.add_text_field(field_name, STRING | STORED);
                                field_map.insert(field_name.clone(), field);
                            } else {
                                let field = schema_builder.add_text_field(field_name, STRING);
                                field_map.insert(field_name.clone(), field);
                            }
                            continue;
                        }

                        let text_indexing = match tokenizer.as_str() {
                            "simple" => TextFieldIndexing::default()
                                .set_tokenizer("simple")
                                .set_index_option(
                                    tantivy::schema::IndexRecordOption::WithFreqsAndPositions,
                                ),
                            "en_stem" => TextFieldIndexing::default()
                                .set_tokenizer("en_stem")
                                .set_index_option(
                                    tantivy::schema::IndexRecordOption::WithFreqsAndPositions,
                                ),
                            _ => TextFieldIndexing::default()
                                .set_tokenizer("default")
                                .set_index_option(
                                    tantivy::schema::IndexRecordOption::WithFreqsAndPositions,
                                ),
                        };

                        options = options.set_indexing_options(text_indexing);
                    }

                    schema_builder.add_text_field(field_name, options)
                }

                FieldType::I64 {
                    stored,
                    indexed,
                    fast,
                } => {
                    let mut options = NumericOptions::default();

                    if *stored {
                        options = options.set_stored();
                    }

                    if *indexed {
                        options = options.set_indexed();
                    }

                    if *fast {
                        options = options.set_fast();
                    }

                    schema_builder.add_i64_field(field_name, options)
                }

                FieldType::F64 {
                    stored,
                    indexed,
                    fast,
                } => {
                    let mut options = NumericOptions::default(); // Note: Tantivy uses NumericOptions for f64 too

                    if *stored {
                        options = options.set_stored();
                    }

                    if *indexed {
                        options = options.set_indexed();
                    }

                    if *fast {
                        options = options.set_fast();
                    }

                    schema_builder.add_f64_field(field_name, options)
                }

                FieldType::Date {
                    stored,
                    indexed,
                    fast,
                } => {
                    let mut options = DateOptions::default();

                    if *stored {
                        options = options.set_stored();
                    }

                    if *indexed {
                        options = options.set_indexed();
                    }

                    if *fast {
                        options = options.set_fast();
                    }

                    schema_builder.add_date_field(field_name, options)
                }

                FieldType::Facet => schema_builder.add_facet_field(field_name, INDEXED),

                FieldType::Bytes { stored, indexed } => {
                    let mut options = tantivy::schema::BytesOptions::default();

                    if *stored {
                        options = options.set_stored();
                    }

                    if *indexed {
                        options = options.set_indexed();
                    }

                    schema_builder.add_bytes_field(field_name, options)
                }

                FieldType::Geo {
                    stored: _,
                    indexed: _,
                } => {
                    // TODO: Implement geospatial fields when Tantivy supports them
                    // For now, we'll skip these fields
                    continue;
                }
            };

            field_map.insert(field_name.clone(), field);
        }

        let schema = schema_builder.build();
        Ok((schema, field_map))
    }

    /// Get the Tantivy schema
    pub fn tantivy_schema(&self) -> &Schema {
        &self.tantivy_schema
    }

    /// Get the schema definition
    pub fn schema_definition(&self) -> &SchemaDefinition {
        &self.schema_def
    }

    /// Get field by name
    pub fn get_field(&self, field_name: &str) -> Option<Field> {
        self.field_map.get(field_name).copied()
    }

    /// Get all fields
    pub fn get_all_fields(&self) -> &HashMap<String, Field> {
        &self.field_map
    }

    /// Convert field value to Tantivy value
    pub fn field_value_to_tantivy(
        &self,
        field_name: &str,
        value: &FieldValue,
    ) -> Result<tantivy::schema::OwnedValue> {
        // let field = self.get_field(field_name).ok_or_else(|| {
        //     SearchEngineError::SchemaError(format!("Field '{}' not found in schema", field_name))
        // })?;
        // Validate field value against schema
        self.validate_field_value(field_name, value)?;

        let tantivy_value = match value {
            FieldValue::Text(text) => tantivy::schema::OwnedValue::Str(text.to_string()),
            FieldValue::I64(num) => tantivy::schema::OwnedValue::I64(*num),
            FieldValue::F64(num) => tantivy::schema::OwnedValue::F64(*num),
            FieldValue::Date(date) => {
                let timestamp = date.timestamp();
                tantivy::schema::OwnedValue::Date(tantivy::DateTime::from_timestamp_secs(timestamp))
            }
            FieldValue::Facet(facet) => {
                let facet_path = tantivy::schema::Facet::from_text(&facet).map_err(|e| {
                    SearchEngineError::SchemaError(format!("Invalid facet '{}': {}", facet, e))
                })?;
                tantivy::schema::OwnedValue::Facet(facet_path)
            }
            FieldValue::Bytes(bytes) => tantivy::schema::OwnedValue::Bytes(bytes.to_vec()),
        };

        Ok(tantivy_value)
    }

    /// Convert Tantivy document to our format
    pub fn document_from_tantivy(
        &self,
        doc: &impl tantivy::Document,
    ) -> Result<HashMap<String, FieldValue>> {
        let mut fields = HashMap::new();

        for (field_name, field) in &self.field_map {
            // Collect all values for this field from the document
            let mut values = Vec::new();
            for (_field, value) in doc.iter_fields_and_values() {
                if _field == *field {
                    values.push(value);
                }
            }

            if !values.is_empty() {
                if let Some(value) = values.first() {
                    // Use pattern matching without trying to match on the enum variant directly
                    let field_value = if let Some(s) = value.as_str() {
                        FieldValue::Text(s.to_string())
                    } else if let Some(i) = value.as_i64() {
                        FieldValue::I64(i)
                    } else if let Some(f) = value.as_f64() {
                        FieldValue::F64(f)
                    } else if let Some(d) = value.as_datetime() {
                        let timestamp = d.into_timestamp_secs();
                        let dt = chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_default();
                        FieldValue::Date(dt)
                    } else if let Some(f) = value.as_facet() {
                        FieldValue::Facet(f.to_string())
                    } else if let Some(b) = value.as_bytes() {
                        FieldValue::Bytes(b.to_vec())
                    } else {
                        continue;
                    };
                    fields.insert(field_name.clone(), field_value);
                }
            }
        }
        Ok(fields)
    }

    /// Validate field value against schema
    pub fn validate_field_value(&self, field_name: &str, value: &FieldValue) -> Result<()> {
        let field_type = self.schema_def.fields.get(field_name).ok_or_else(|| {
            SearchEngineError::SchemaError(format!("Field '{}' not found in schema", field_name))
        })?;

        let is_valid = match (field_type, value) {
            (FieldType::Text { .. }, FieldValue::Text(_)) => true,
            (FieldType::I64 { .. }, FieldValue::I64(_)) => true,
            (FieldType::F64 { .. }, FieldValue::F64(_)) => true,
            (FieldType::Date { .. }, FieldValue::Date(_)) => true,
            (FieldType::Facet, FieldValue::Facet(_)) => true,
            (FieldType::Bytes { .. }, FieldValue::Bytes(_)) => true,
            _ => false,
        };

        if !is_valid {
            return Err(SearchEngineError::SchemaError(format!(
                "Field '{}' type mismatch. Expected {:?}, got {:?}",
                field_name, field_type, value
            )));
        }

        Ok(())
    }
}
