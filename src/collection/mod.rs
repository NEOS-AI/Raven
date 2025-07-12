use crate::error::{Result, SearchEngineError};
use crate::schema::SchemaManager;
use crate::types::{CollectionStats, FieldValue, IndexDocument, SchemaDefinition};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tantivy::{Index, IndexWriter, ReloadPolicy, doc};

/// Collection represents a single searchable collection with its own schema
#[derive(Clone)]
pub struct Collection {
    pub name: String,
    pub schema_manager: Arc<SchemaManager>,
    pub index: Index,
    pub writer: Arc<RwLock<IndexWriter>>,
    pub data_path: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl Collection {
    /// Create a new collection with the given schema
    pub fn create<P: AsRef<Path>>(
        name: String,
        schema_def: SchemaDefinition,
        data_dir: P,
        heap_size: usize,
    ) -> Result<Self> {
        let schema_manager = Arc::new(SchemaManager::new(schema_def)?);
        let collection_path = data_dir.as_ref().join(&name);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&collection_path)?;

        // Create Tantivy index
        let index =
            Index::create_in_dir(&collection_path, schema_manager.tantivy_schema().clone())?;

        // Create index writer
        let writer = index.writer(heap_size)?;

        let now = Utc::now();

        let collection = Self {
            name,
            schema_manager,
            index,
            writer: Arc::new(RwLock::new(writer)),
            data_path: collection_path,
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
        };

        // Save schema definition to disk
        collection.save_schema_definition()?;

        Ok(collection)
    }

    /// Open an existing collection
    pub fn open<P: AsRef<Path>>(name: String, data_dir: P, heap_size: usize) -> Result<Self> {
        let collection_path = data_dir.as_ref().join(&name);

        if !collection_path.exists() {
            return Err(SearchEngineError::CollectionError(format!(
                "Collection '{}' does not exist",
                name
            )));
        }

        // Load schema definition
        let schema_def = Self::load_schema_definition(&collection_path)?;
        let schema_manager = Arc::new(SchemaManager::new(schema_def)?);

        // Open Tantivy index
        let index = Index::open_in_dir(&collection_path)?;

        // Create index writer
        let writer = index.writer(heap_size)?;

        // Load metadata
        let metadata = Self::load_metadata(&collection_path)?;

        Ok(Self {
            name,
            schema_manager,
            index,
            writer: Arc::new(RwLock::new(writer)),
            data_path: collection_path,
            created_at: metadata.created_at,
            updated_at: Arc::new(RwLock::new(metadata.updated_at)),
        })
    }

    /// Add a document to the collection
    pub fn add_document(&self, doc: IndexDocument) -> Result<()> {
        let mut tantivy_doc = tantivy::schema::document::TantivyDocument::default();

        // Add document ID
        let id_field = self
            .schema_manager
            .get_field("_id")
            .ok_or_else(|| SearchEngineError::IndexError("ID field not found".to_string()))?;
        tantivy_doc.add_text(id_field, doc.id.clone());

        // Add document fields
        for (field_name, field_value) in &doc.fields {
            // Validate field value
            self.schema_manager
                .validate_field_value(field_name, field_value)?;

            let field = self.schema_manager.get_field(field_name).ok_or_else(|| {
                SearchEngineError::SchemaError(format!(
                    "Field '{}' not found in schema",
                    field_name
                ))
            })?;

            match field_value {
                FieldValue::Text(s) => tantivy_doc.add_text(field, s),
                FieldValue::I64(i) => tantivy_doc.add_i64(field, *i),
                FieldValue::F64(f) => tantivy_doc.add_f64(field, *f),
                FieldValue::Date(d) => tantivy_doc
                    .add_date(field, tantivy::DateTime::from_timestamp_secs(d.timestamp())),
                FieldValue::Facet(f) => {
                    let facet = tantivy::schema::Facet::from_text(f).map_err(|e| {
                        SearchEngineError::IndexError(format!("Invalid facet '{}': {}", f, e))
                    })?;
                    tantivy_doc.add_facet(field, facet)
                }
                FieldValue::Bytes(b) => tantivy_doc.add_bytes(field, b),
                // _ => {
                //     return Err(SearchEngineError::IndexError(format!(
                //         "Unsupported value type for field '{}'",
                //         field_name
                //     )));
                // }
            }
        }

        // Add document to index
        {
            let writer = self.writer.write().unwrap();
            writer.add_document(tantivy_doc)?;
        }

        // Update timestamp
        *self.updated_at.write().unwrap() = Utc::now();

        Ok(())
    }

    /// Update a document by ID
    pub fn update_document(&self, doc: IndexDocument) -> Result<()> {
        let id_field = self
            .schema_manager
            .get_field("_id")
            .ok_or_else(|| SearchEngineError::IndexError("ID field not found".to_string()))?;

        let term = tantivy::Term::from_field_text(id_field, &doc.id);

        let mut tantivy_doc = tantivy::schema::document::TantivyDocument::default();
        tantivy_doc.add_text(id_field, doc.id.clone());

        // Add document fields
        for (field_name, field_value) in &doc.fields {
            self.schema_manager
                .validate_field_value(field_name, field_value)?;

            let field = self.schema_manager.get_field(field_name).ok_or_else(|| {
                SearchEngineError::SchemaError(format!(
                    "Field '{}' not found in schema",
                    field_name
                ))
            })?;

            let tantivy_value = self
                .schema_manager
                .field_value_to_tantivy(field_name, field_value)?;

            match tantivy_value {
                tantivy::schema::OwnedValue::Str(s) => tantivy_doc.add_text(field, s),
                tantivy::schema::OwnedValue::I64(i) => tantivy_doc.add_i64(field, i),
                tantivy::schema::OwnedValue::F64(f) => tantivy_doc.add_f64(field, f),
                tantivy::schema::OwnedValue::Date(d) => tantivy_doc.add_date(field, d),
                tantivy::schema::OwnedValue::Facet(f) => tantivy_doc.add_facet(field, f),
                tantivy::schema::OwnedValue::Bytes(b) => tantivy_doc.add_bytes(field, &b),
                _ => {
                    return Err(SearchEngineError::IndexError(format!(
                        "Unsupported value type for field '{}'",
                        field_name
                    )));
                }
            }
        }

        // Update document in index
        {
            let writer = self.writer.write().unwrap();
            writer.delete_term(term);
            writer.add_document(tantivy_doc)?;
        }

        // Update timestamp
        *self.updated_at.write().unwrap() = Utc::now();

        Ok(())
    }

    /// Delete a document by ID
    pub fn delete_document(&self, doc_id: &str) -> Result<()> {
        let id_field = self
            .schema_manager
            .get_field("_id")
            .ok_or_else(|| SearchEngineError::IndexError("ID field not found".to_string()))?;

        let term = tantivy::Term::from_field_text(id_field, doc_id);

        {
            let writer = self.writer.write().unwrap();
            writer.delete_term(term);
        }

        // Update timestamp
        *self.updated_at.write().unwrap() = Utc::now();

        Ok(())
    }

    /// Commit changes to the index
    pub fn commit(&self) -> Result<()> {
        {
            let mut writer = self.writer.write().unwrap();
            writer.commit()?;
        }

        // Reload searcher
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        reader.reload()?;

        // Update timestamp and save metadata
        *self.updated_at.write().unwrap() = Utc::now();
        self.save_metadata()?;

        Ok(())
    }

    /// Get collection statistics
    pub fn get_stats(&self) -> Result<CollectionStats> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let num_docs = searcher.num_docs() as usize;

        // Calculate index size (approximate)
        let index_size = self.calculate_index_size()?;

        Ok(CollectionStats {
            name: self.name.clone(),
            document_count: num_docs,
            index_size_bytes: index_size,
            created_at: self.created_at,
            updated_at: *self.updated_at.read().unwrap(),
        })
    }

    /// Save schema definition to disk
    fn save_schema_definition(&self) -> Result<()> {
        let schema_path = self.data_path.join("schema.json");
        let schema_json = serde_json::to_string_pretty(self.schema_manager.schema_definition())?;
        std::fs::write(schema_path, schema_json)?;
        Ok(())
    }

    /// Load schema definition from disk
    fn load_schema_definition<P: AsRef<Path>>(collection_path: P) -> Result<SchemaDefinition> {
        let schema_path = collection_path.as_ref().join("schema.json");
        let schema_json = std::fs::read_to_string(schema_path)?;
        let schema_def: SchemaDefinition = serde_json::from_str(&schema_json)?;
        Ok(schema_def)
    }

    /// Save metadata to disk
    fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.data_path.join("metadata.json");
        let metadata = CollectionMetadata {
            name: self.name.clone(),
            created_at: self.created_at,
            updated_at: *self.updated_at.read().unwrap(),
        };
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(metadata_path, metadata_json)?;
        Ok(())
    }

    /// Load metadata from disk
    fn load_metadata<P: AsRef<Path>>(collection_path: P) -> Result<CollectionMetadata> {
        let metadata_path = collection_path.as_ref().join("metadata.json");

        if !metadata_path.exists() {
            // Create default metadata if not exists
            let now = Utc::now();
            return Ok(CollectionMetadata {
                name: collection_path
                    .as_ref()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                created_at: now,
                updated_at: now,
            });
        }

        let metadata_json = std::fs::read_to_string(metadata_path)?;
        let metadata: CollectionMetadata = serde_json::from_str(&metadata_json)?;
        Ok(metadata)
    }

    /// Calculate approximate index size
    fn calculate_index_size(&self) -> Result<u64> {
        fn dir_size(path: &Path) -> std::io::Result<u64> {
            let mut size = 0;
            if path.is_dir() {
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        size += entry.metadata()?.len();
                    } else if path.is_dir() {
                        size += dir_size(&path)?;
                    }
                }
            } else {
                size = std::fs::metadata(path)?.len();
            }
            Ok(size)
        }

        let total_size = dir_size(&self.data_path)?;
        Ok(total_size)
    }
}

/// Internal metadata structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CollectionMetadata {
    name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}
