//! # Rust Search Engine
//!
//! A high-performance search engine built with Rust and Tantivy, supporting:
//! - Full-text search with BM25 and TF-IDF scoring
//! - Numeric range queries with B-Tree indexing
//! - Custom schema definitions per collection
//! - Modular architecture for extensibility
//! - Future support for geospatial indexing

pub mod collection;
pub mod engine;
pub mod error;
pub mod schema;
pub mod search;
pub mod types;

// Re-export commonly used types
pub use engine::{CollectionHealth, EngineHealth, RustSearchEngine};
pub use error::{Result, SearchEngineError};
pub use types::{
    CollectionStats, EngineConfig, FieldType, FieldValue, IndexDocument, QueryExpression,
    SchemaDefinition, SearchHit, SearchQuery, SearchResult, SortField, SortOrder,
};

/// Convenience function to create a new search engine with default configuration
pub fn create_engine() -> Result<RustSearchEngine> {
    let config = EngineConfig::default();
    RustSearchEngine::new(config)
}

/// Convenience function to create a search engine with custom data directory
pub fn create_engine_with_data_dir<P: AsRef<std::path::Path>>(
    data_dir: P,
) -> Result<RustSearchEngine> {
    let mut config = EngineConfig::default();
    config.data_dir = data_dir.as_ref().to_string_lossy().to_string();
    RustSearchEngine::new(config)
}

/// Builder pattern for creating engine configurations
pub struct EngineConfigBuilder {
    config: EngineConfig,
}

impl EngineConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    pub fn data_dir<P: AsRef<std::path::Path>>(mut self, data_dir: P) -> Self {
        self.config.data_dir = data_dir.as_ref().to_string_lossy().to_string();
        self
    }

    pub fn heap_size(mut self, heap_size: usize) -> Self {
        self.config.default_heap_size = heap_size;
        self
    }

    pub fn commit_interval_ms(mut self, interval_ms: u64) -> Self {
        self.config.commit_interval_ms = interval_ms;
        self
    }

    pub fn enable_compression(mut self, enable: bool) -> Self {
        self.config.enable_compression = enable;
        self
    }

    pub fn build(self) -> EngineConfig {
        self.config
    }
}

impl Default for EngineConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating common schema definitions
pub mod schema_helpers {
    use super::types::{FieldType, SchemaDefinition};
    use std::collections::HashMap;

    /// Create a simple text collection schema
    pub fn text_collection_schema(name: &str, fields: &[(&str, bool, bool)]) -> SchemaDefinition {
        let mut field_map = HashMap::new();

        for (field_name, stored, indexed) in fields {
            field_map.insert(
                field_name.to_string(),
                FieldType::Text {
                    stored: *stored,
                    indexed: *indexed,
                    tokenizer: "default".to_string(),
                },
            );
        }

        SchemaDefinition {
            name: name.to_string(),
            fields: field_map,
            primary_key: None,
        }
    }

    /// Create a blog post collection schema
    pub fn blog_post_schema() -> SchemaDefinition {
        let mut fields = HashMap::new();

        fields.insert(
            "title".to_string(),
            FieldType::Text {
                stored: true,
                indexed: true,
                tokenizer: "default".to_string(),
            },
        );

        fields.insert(
            "content".to_string(),
            FieldType::Text {
                stored: true,
                indexed: true,
                tokenizer: "default".to_string(),
            },
        );

        fields.insert(
            "author".to_string(),
            FieldType::Text {
                stored: true,
                indexed: true,
                tokenizer: "keyword".to_string(),
            },
        );

        fields.insert(
            "published_date".to_string(),
            FieldType::Date {
                stored: true,
                indexed: true,
                fast: true,
            },
        );

        fields.insert(
            "view_count".to_string(),
            FieldType::I64 {
                stored: true,
                indexed: true,
                fast: true,
            },
        );

        fields.insert(
            "rating".to_string(),
            FieldType::F64 {
                stored: true,
                indexed: true,
                fast: true,
            },
        );

        fields.insert("category".to_string(), FieldType::Facet);

        SchemaDefinition {
            name: "blog_posts".to_string(),
            fields,
            primary_key: Some("_id".to_string()),
        }
    }

    /// Create a product catalog schema
    pub fn product_catalog_schema() -> SchemaDefinition {
        let mut fields = HashMap::new();

        fields.insert(
            "name".to_string(),
            FieldType::Text {
                stored: true,
                indexed: true,
                tokenizer: "default".to_string(),
            },
        );

        fields.insert(
            "description".to_string(),
            FieldType::Text {
                stored: true,
                indexed: true,
                tokenizer: "default".to_string(),
            },
        );

        fields.insert(
            "price".to_string(),
            FieldType::F64 {
                stored: true,
                indexed: true,
                fast: true,
            },
        );

        fields.insert(
            "stock_quantity".to_string(),
            FieldType::I64 {
                stored: true,
                indexed: true,
                fast: true,
            },
        );

        fields.insert(
            "brand".to_string(),
            FieldType::Text {
                stored: true,
                indexed: true,
                tokenizer: "keyword".to_string(),
            },
        );

        fields.insert("category".to_string(), FieldType::Facet);

        SchemaDefinition {
            name: "products".to_string(),
            fields,
            primary_key: Some("_id".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_engine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let engine = create_engine_with_data_dir(temp_dir.path()).unwrap();

        // Test basic engine functionality
        let collections = engine.list_collections();
        assert!(collections.is_empty());
    }

    #[test]
    fn test_schema_builder() {
        let schema = schema_helpers::blog_post_schema();
        assert_eq!(schema.name, "blog_posts");
        assert!(schema.fields.contains_key("title"));
        assert!(schema.fields.contains_key("content"));
        assert!(schema.fields.contains_key("published_date"));
    }

    #[test]
    fn test_config_builder() {
        let config = EngineConfigBuilder::new()
            .data_dir("/tmp/test")
            .heap_size(100_000_000)
            .commit_interval_ms(5000)
            .enable_compression(false)
            .build();

        assert_eq!(config.data_dir, "/tmp/test");
        assert_eq!(config.default_heap_size, 100_000_000);
        assert_eq!(config.commit_interval_ms, 5000);
        assert!(!config.enable_compression);
    }
}
