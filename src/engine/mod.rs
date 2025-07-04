use crate::collection::Collection;
use crate::error::{Result, SearchEngineError};
use crate::search::SearchEngine;
use crate::types::{
    CollectionStats, EngineConfig, IndexDocument, SchemaDefinition, SearchQuery, SearchResult,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, interval};

/// Main search engine that manages multiple collections
pub struct RustSearchEngine {
    config: EngineConfig,
    collections: Arc<RwLock<HashMap<String, Collection>>>,
    auto_commit_handle: Option<tokio::task::JoinHandle<()>>,
}

impl RustSearchEngine {
    /// Create a new search engine with the given configuration
    pub fn new(config: EngineConfig) -> Result<Self> {
        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&config.data_dir)?;

        let collections = Arc::new(RwLock::new(HashMap::new()));

        let mut engine = Self {
            config,
            collections,
            auto_commit_handle: None,
        };

        // Load existing collections
        engine.load_existing_collections()?;

        Ok(engine)
    }

    /// Start the search engine with auto-commit functionality
    pub async fn start(&mut self) -> Result<()> {
        // Start auto-commit task
        let collections = self.collections.clone();
        let commit_interval = self.config.commit_interval_ms;

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(commit_interval));

            loop {
                interval.tick().await;

                // Commit all collections
                let collections_guard = collections.read().unwrap();
                for collection in collections_guard.values() {
                    if let Err(e) = collection.commit() {
                        tracing::warn!(
                            "Failed to auto-commit collection '{}': {}",
                            collection.name,
                            e
                        );
                    }
                }
            }
        });

        self.auto_commit_handle = Some(handle);

        tracing::info!(
            "Search engine started with auto-commit interval: {}ms",
            commit_interval
        );
        Ok(())
    }

    /// Stop the search engine
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.auto_commit_handle.take() {
            handle.abort();
        }

        // Final commit for all collections
        self.commit_all().await?;

        tracing::info!("Search engine stopped");
        Ok(())
    }

    /// Create a new collection with the given schema
    pub fn create_collection(&self, name: String, schema_def: SchemaDefinition) -> Result<()> {
        let mut collections = self.collections.write().unwrap();

        if collections.contains_key(&name) {
            return Err(SearchEngineError::CollectionError(format!(
                "Collection '{}' already exists",
                name
            )));
        }

        let collection = Collection::create(
            name.clone(),
            schema_def,
            &self.config.data_dir,
            self.config.default_heap_size,
        )?;

        collections.insert(name.clone(), collection);

        tracing::info!("Created collection: {}", name);
        Ok(())
    }

    /// Drop a collection
    pub fn drop_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.write().unwrap();

        if let Some(collection) = collections.remove(name) {
            // Commit final changes
            collection.commit()?;

            // Remove collection directory
            let collection_path = Path::new(&self.config.data_dir).join(name);
            if collection_path.exists() {
                std::fs::remove_dir_all(collection_path)?;
            }

            tracing::info!("Dropped collection: {}", name);
            Ok(())
        } else {
            Err(SearchEngineError::CollectionError(format!(
                "Collection '{}' not found",
                name
            )))
        }
    }

    /// List all collections
    pub fn list_collections(&self) -> Vec<String> {
        let collections = self.collections.read().unwrap();
        collections.keys().cloned().collect()
    }

    /// Get collection statistics
    pub fn get_collection_stats(&self, name: &str) -> Result<CollectionStats> {
        let collections = self.collections.read().unwrap();
        let collection = collections.get(name).ok_or_else(|| {
            SearchEngineError::CollectionError(format!("Collection '{}' not found", name))
        })?;

        collection.get_stats()
    }

    /// Get statistics for all collections
    pub fn get_all_stats(&self) -> Result<Vec<CollectionStats>> {
        let collections = self.collections.read().unwrap();
        let mut stats = Vec::new();

        for collection in collections.values() {
            stats.push(collection.get_stats()?);
        }

        Ok(stats)
    }

    /// Add a document to a collection
    pub fn add_document(&self, collection_name: &str, doc: IndexDocument) -> Result<()> {
        let collections = self.collections.read().unwrap();
        let collection = collections.get(collection_name).ok_or_else(|| {
            SearchEngineError::CollectionError(format!(
                "Collection '{}' not found",
                collection_name
            ))
        })?;

        collection.add_document(doc)?;

        tracing::debug!("Added document to collection: {}", collection_name);
        Ok(())
    }

    /// Update a document in a collection
    pub fn update_document(&self, collection_name: &str, doc: IndexDocument) -> Result<()> {
        let collections = self.collections.read().unwrap();
        let collection = collections.get(collection_name).ok_or_else(|| {
            SearchEngineError::CollectionError(format!(
                "Collection '{}' not found",
                collection_name
            ))
        })?;

        collection.update_document(doc)?;

        tracing::debug!("Updated document in collection: {}", collection_name);
        Ok(())
    }

    /// Delete a document from a collection
    pub fn delete_document(&self, collection_name: &str, doc_id: &str) -> Result<()> {
        let collections = self.collections.read().unwrap();
        let collection = collections.get(collection_name).ok_or_else(|| {
            SearchEngineError::CollectionError(format!(
                "Collection '{}' not found",
                collection_name
            ))
        })?;

        collection.delete_document(doc_id)?;

        tracing::debug!("Deleted document from collection: {}", collection_name);
        Ok(())
    }

    /// Search documents in a collection
    pub fn search(&self, query: SearchQuery) -> Result<SearchResult> {
        let collections = self.collections.read().unwrap();
        let collection = collections.get(&query.collection).ok_or_else(|| {
            SearchEngineError::CollectionError(format!(
                "Collection '{}' not found",
                query.collection
            ))
        })?;

        let search_engine = SearchEngine::new(collection.clone());
        let result = search_engine.search(query)?;

        tracing::debug!("Search completed in {}ms", result.took_ms);
        Ok(result)
    }

    /// Commit changes for a specific collection
    pub fn commit_collection(&self, collection_name: &str) -> Result<()> {
        let collections = self.collections.read().unwrap();
        let collection = collections.get(collection_name).ok_or_else(|| {
            SearchEngineError::CollectionError(format!(
                "Collection '{}' not found",
                collection_name
            ))
        })?;

        collection.commit()?;

        tracing::debug!("Committed collection: {}", collection_name);
        Ok(())
    }

    /// Commit changes for all collections
    pub async fn commit_all(&self) -> Result<()> {
        let collections = self.collections.read().unwrap();

        for (name, collection) in collections.iter() {
            if let Err(e) = collection.commit() {
                tracing::error!("Failed to commit collection '{}': {}", name, e);
                return Err(e);
            }
        }

        tracing::debug!("Committed all collections");
        Ok(())
    }

    /// Load existing collections from disk
    fn load_existing_collections(&mut self) -> Result<()> {
        let data_dir = Path::new(&self.config.data_dir);

        if !data_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(data_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let collection_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();

                // Check if this is a valid collection directory
                let schema_path = path.join("schema.json");
                if schema_path.exists() {
                    match Collection::open(
                        collection_name.clone(),
                        &self.config.data_dir,
                        self.config.default_heap_size,
                    ) {
                        Ok(collection) => {
                            let mut collections = self.collections.write().unwrap();
                            collections.insert(collection_name.clone(), collection);
                            tracing::info!("Loaded existing collection: {}", collection_name);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load collection '{}': {}",
                                collection_name,
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get engine configuration
    pub fn get_config(&self) -> &EngineConfig {
        &self.config
    }

    /// Update engine configuration (some settings require restart)
    pub fn update_config(&mut self, new_config: EngineConfig) -> Result<()> {
        // Validate new configuration
        if new_config.data_dir != self.config.data_dir {
            return Err(SearchEngineError::ConfigError(
                "Cannot change data directory while engine is running".to_string(),
            ));
        }

        self.config = new_config;
        tracing::info!("Updated engine configuration");
        Ok(())
    }

    /// Health check for the search engine
    pub fn health_check(&self) -> Result<EngineHealth> {
        let collections = self.collections.read().unwrap();
        let mut collection_healths = Vec::new();

        for (name, collection) in collections.iter() {
            let stats = collection.get_stats()?;
            collection_healths.push(CollectionHealth {
                name: name.clone(),
                status: "healthy".to_string(),
                document_count: stats.document_count,
                index_size_bytes: stats.index_size_bytes,
            });
        }

        Ok(EngineHealth {
            status: "healthy".to_string(),
            collections: collection_healths,
            uptime_ms: 0, // TODO: Track actual uptime
        })
    }
}

/// Engine health information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineHealth {
    pub status: String,
    pub collections: Vec<CollectionHealth>,
    pub uptime_ms: u64,
}

/// Collection health information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CollectionHealth {
    pub name: String,
    pub status: String,
    pub document_count: usize,
    pub index_size_bytes: u64,
}

impl Drop for RustSearchEngine {
    fn drop(&mut self) {
        if let Some(handle) = self.auto_commit_handle.take() {
            handle.abort();
        }

        // Final commit for all collections
        let collections = self.collections.read().unwrap();
        for (name, collection) in collections.iter() {
            if let Err(e) = collection.commit() {
                tracing::error!(
                    "Failed to commit collection '{}' during shutdown: {}",
                    name,
                    e
                );
            }
        }
    }
}
