use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tantivy::Score;

/// Field type definitions for schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    /// Text field for full-text search
    Text {
        stored: bool,
        indexed: bool,
        tokenizer: String,
    },
    /// Integer field for numeric search
    I64 {
        stored: bool,
        indexed: bool,
        fast: bool, // For range queries
    },
    /// Float field for numeric search
    F64 {
        stored: bool,
        indexed: bool,
        fast: bool,
    },
    /// Date field
    Date {
        stored: bool,
        indexed: bool,
        fast: bool,
    },
    /// Facet field for categorical data
    Facet,
    /// Binary field for raw data
    Bytes { stored: bool, indexed: bool },
    /// Future: Geospatial field
    #[allow(dead_code)]
    Geo { stored: bool, indexed: bool },
}

/// Schema definition for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub name: String,
    pub fields: HashMap<String, FieldType>,
    pub primary_key: Option<String>,
}

/// Document to be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocument {
    pub id: String,
    pub fields: HashMap<String, FieldValue>,
}

/// Field value enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldValue {
    Text(String),
    I64(i64),
    F64(f64),
    Date(chrono::DateTime<chrono::Utc>),
    Facet(String),
    Bytes(Vec<u8>),
}

/// Search query definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub collection: String,
    pub query: QueryExpression,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort: Option<Vec<SortField>>,
}

/// Query expression enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryExpression {
    /// Full-text query
    FullText {
        field: String,
        text: String,
        boost: Option<f32>,
    },
    /// Term query for exact match
    Term { field: String, value: FieldValue },
    /// Range query for numeric fields
    Range {
        field: String,
        min: Option<FieldValue>,
        max: Option<FieldValue>,
        inclusive: bool,
    },
    /// Boolean query combining multiple queries
    Bool {
        must: Option<Vec<QueryExpression>>,
        should: Option<Vec<QueryExpression>>,
        must_not: Option<Vec<QueryExpression>>,
        minimum_should_match: Option<usize>,
    },
    /// Match all documents
    MatchAll,
}

/// Sort field specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortField {
    pub field: String,
    pub order: SortOrder,
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub total_hits: usize,
    pub documents: Vec<SearchHit>,
    pub took_ms: u64,
}

/// Individual search hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub score: Score,
    pub fields: HashMap<String, FieldValue>,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub name: String,
    pub document_count: usize,
    pub index_size_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub data_dir: String,
    pub default_heap_size: usize,
    pub commit_interval_ms: u64,
    pub enable_compression: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            default_heap_size: 50_000_000, // 50MB
            commit_interval_ms: 1000,      // 1 second
            enable_compression: true,
        }
    }
}
