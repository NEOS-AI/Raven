use clap::{Parser, Subcommand};
use raven::{
    EngineConfigBuilder, FieldType, FieldValue, IndexDocument, QueryExpression, RustSearchEngine,
    SchemaDefinition, SearchQuery, schema_helpers,
};
use serde_json;
use std::collections::HashMap;
use std::io::{self, Write};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "raven")]
#[command(about = "A high-performance search engine built with Rust and Tantivy")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "./data")]
    data_dir: String,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new collection
    CreateCollection {
        /// Collection name
        name: String,
        /// Schema file path (JSON)
        #[arg(short, long)]
        schema: Option<String>,
        /// Use predefined schema type (blog_post, product_catalog)
        #[arg(short, long)]
        template: Option<String>,
    },

    /// List all collections
    ListCollections,

    /// Drop a collection
    DropCollection {
        /// Collection name
        name: String,
    },

    /// Add a document to a collection
    AddDocument {
        /// Collection name
        collection: String,
        /// Document JSON file path
        #[arg(short, long)]
        file: Option<String>,
        /// Document JSON string
        #[arg(short, long)]
        json: Option<String>,
    },

    /// Search documents
    Search {
        /// Collection name
        collection: String,
        /// Search query
        query: String,
        /// Field to search (for full-text search)
        #[arg(short, long, default_value = "content")]
        field: String,
        /// Number of results to return
        #[arg(short, long, default_value = "10")]
        limit: usize,
        /// Number of results to skip
        #[arg(short, long, default_value = "0")]
        offset: usize,
    },

    /// Get collection statistics
    Stats {
        /// Collection name (optional, shows all if not specified)
        collection: Option<String>,
    },

    /// Start interactive mode
    Interactive,

    /// Health check
    Health,

    /// Commit changes
    Commit {
        /// Collection name (optional, commits all if not specified)
        collection: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Create engine
    let config = EngineConfigBuilder::new().data_dir(&cli.data_dir).build();

    let mut engine = RustSearchEngine::new(config)?;
    engine.start().await?;

    match cli.command {
        Commands::CreateCollection {
            name,
            schema,
            template,
        } => {
            let schema_def = if let Some(schema_path) = schema {
                // Load schema from file
                let schema_content = std::fs::read_to_string(schema_path)?;
                serde_json::from_str(&schema_content)?
            } else if let Some(template_name) = template {
                // Use predefined template
                match template_name.as_str() {
                    "blog_post" => schema_helpers::blog_post_schema(),
                    "product_catalog" => schema_helpers::product_catalog_schema(),
                    _ => {
                        eprintln!(
                            "Unknown template: {}. Available templates: blog_post, product_catalog",
                            template_name
                        );
                        std::process::exit(1);
                    }
                }
            } else {
                // Use interactive schema creation
                create_schema_interactively(&name)?
            };

            engine.create_collection(name.clone(), schema_def)?;
            println!("Created collection: {}", name);
        }

        Commands::ListCollections => {
            let collections = engine.list_collections();
            if collections.is_empty() {
                println!("No collections found");
            } else {
                println!("Collections:");
                for collection in collections {
                    println!("  - {}", collection);
                }
            }
        }

        Commands::DropCollection { name } => {
            engine.drop_collection(&name)?;
            println!("Dropped collection: {}", name);
        }

        Commands::AddDocument {
            collection,
            file,
            json,
        } => {
            let document_json = if let Some(file_path) = file {
                std::fs::read_to_string(file_path)?
            } else if let Some(json_str) = json {
                json_str
            } else {
                eprintln!("Either --file or --json must be specified");
                std::process::exit(1);
            };

            let document: IndexDocument = serde_json::from_str(&document_json)?;
            engine.add_document(&collection, document)?;
            println!("Added document to collection: {}", collection);
        }

        Commands::Search {
            collection,
            query,
            field,
            limit,
            offset,
        } => {
            let search_query = SearchQuery {
                collection: collection.clone(),
                query: QueryExpression::FullText {
                    field,
                    text: query,
                    boost: None,
                },
                limit: Some(limit),
                offset: Some(offset),
                sort: None,
            };

            let result = engine.search(search_query)?;

            println!("Search Results:");
            println!(
                "Total hits: {} (took {}ms)",
                result.total_hits, result.took_ms
            );
            println!();

            for (i, hit) in result.documents.iter().enumerate() {
                println!(
                    "{}. Document ID: {} (score: {:.4})",
                    i + 1,
                    hit.id,
                    hit.score
                );
                for (field_name, field_value) in &hit.fields {
                    match field_value {
                        FieldValue::Text(text) => {
                            let preview = if text.len() > 100 {
                                format!("{}...", &text[..100])
                            } else {
                                text.clone()
                            };
                            println!("   {}: {}", field_name, preview);
                        }
                        _ => println!("   {}: {:?}", field_name, field_value),
                    }
                }
                println!();
            }
        }

        Commands::Stats { collection } => {
            if let Some(collection_name) = collection {
                let stats = engine.get_collection_stats(&collection_name)?;
                println!("Collection: {}", stats.name);
                println!("Documents: {}", stats.document_count);
                println!("Index size: {} bytes", stats.index_size_bytes);
                println!("Created: {}", stats.created_at);
                println!("Updated: {}", stats.updated_at);
            } else {
                let all_stats = engine.get_all_stats()?;
                if all_stats.is_empty() {
                    println!("No collections found");
                } else {
                    for stats in all_stats {
                        println!("Collection: {}", stats.name);
                        println!("  Documents: {}", stats.document_count);
                        println!("  Index size: {} bytes", stats.index_size_bytes);
                        println!("  Created: {}", stats.created_at);
                        println!("  Updated: {}", stats.updated_at);
                        println!();
                    }
                }
            }
        }

        Commands::Interactive => {
            run_interactive_mode(&mut engine).await?;
        }

        Commands::Health => {
            let health = engine.health_check()?;
            println!("Engine Status: {}", health.status);
            println!("Collections:");
            for collection_health in health.collections {
                println!(
                    "  - {}: {} ({} docs, {} bytes)",
                    collection_health.name,
                    collection_health.status,
                    collection_health.document_count,
                    collection_health.index_size_bytes
                );
            }
        }

        Commands::Commit { collection } => {
            if let Some(collection_name) = collection {
                engine.commit_collection(&collection_name)?;
                println!("Committed collection: {}", collection_name);
            } else {
                engine.commit_all().await?;
                println!("Committed all collections");
            }
        }
    }

    engine.stop().await?;
    Ok(())
}

fn create_schema_interactively(collection_name: &str) -> anyhow::Result<SchemaDefinition> {
    println!("Creating schema for collection: {}", collection_name);
    println!("Enter field definitions (type 'done' when finished):");

    let mut fields = HashMap::new();

    loop {
        print!("Field name (or 'done'): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let field_name = input.trim().to_string();

        if field_name == "done" {
            break;
        }

        if field_name.is_empty() {
            continue;
        }

        println!("Field types: text, i64, f64, date, facet, bytes");
        print!("Field type: ");
        io::stdout().flush()?;

        input.clear();
        io::stdin().read_line(&mut input)?;
        let field_type_str = input.trim();

        let field_type = match field_type_str {
            "text" => {
                print!("Stored (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let stored = input.trim().to_lowercase() == "y";

                print!("Indexed (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let indexed = input.trim().to_lowercase() == "y";

                print!("Tokenizer (default, simple, en_stem, keyword): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let tokenizer = input.trim();
                let tokenizer = if tokenizer.is_empty() {
                    "default"
                } else {
                    tokenizer
                };

                FieldType::Text {
                    stored,
                    indexed,
                    tokenizer: tokenizer.to_string(),
                }
            }
            "i64" => {
                print!("Stored (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let stored = input.trim().to_lowercase() == "y";

                print!("Indexed (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let indexed = input.trim().to_lowercase() == "y";

                print!("Fast (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let fast = input.trim().to_lowercase() == "y";

                FieldType::I64 {
                    stored,
                    indexed,
                    fast,
                }
            }
            "f64" => {
                print!("Stored (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let stored = input.trim().to_lowercase() == "y";

                print!("Indexed (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let indexed = input.trim().to_lowercase() == "y";

                print!("Fast (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let fast = input.trim().to_lowercase() == "y";

                FieldType::F64 {
                    stored,
                    indexed,
                    fast,
                }
            }
            "date" => {
                print!("Stored (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let stored = input.trim().to_lowercase() == "y";

                print!("Indexed (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let indexed = input.trim().to_lowercase() == "y";

                print!("Fast (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let fast = input.trim().to_lowercase() == "y";

                FieldType::Date {
                    stored,
                    indexed,
                    fast,
                }
            }
            "facet" => FieldType::Facet,
            "bytes" => {
                print!("Stored (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let stored = input.trim().to_lowercase() == "y";

                print!("Indexed (y/n): ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let indexed = input.trim().to_lowercase() == "y";

                FieldType::Bytes { stored, indexed }
            }
            _ => {
                println!("Unknown field type: {}", field_type_str);
                continue;
            }
        };

        fields.insert(field_name.to_string(), field_type);
        println!("Added field: {}", field_name);
    }

    Ok(SchemaDefinition {
        name: collection_name.to_string(),
        fields,
        primary_key: Some("_id".to_string()),
    })
}

async fn run_interactive_mode(engine: &mut RustSearchEngine) -> anyhow::Result<()> {
    println!("Rust Search Engine - Interactive Mode");
    println!("Type 'help' for available commands, 'quit' to exit");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];

        match command {
            "help" => {
                println!("Available commands:");
                println!("  help                    - Show this help");
                println!("  quit                    - Exit interactive mode");
                println!("  list                    - List all collections");
                println!("  stats [collection]      - Show collection statistics");
                println!("  search <collection> <query> - Search in collection");
                println!("  commit [collection]     - Commit changes");
                println!("  health                  - Show engine health");
            }
            "quit" => break,
            "list" => {
                let collections = engine.list_collections();
                if collections.is_empty() {
                    println!("No collections found");
                } else {
                    println!("Collections: {}", collections.join(", "));
                }
            }
            "stats" => {
                if parts.len() > 1 {
                    let collection_name = parts[1];
                    match engine.get_collection_stats(collection_name) {
                        Ok(stats) => {
                            println!(
                                "Collection: {} ({} docs, {} bytes)",
                                stats.name, stats.document_count, stats.index_size_bytes
                            );
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    match engine.get_all_stats() {
                        Ok(all_stats) => {
                            for stats in all_stats {
                                println!(
                                    "Collection: {} ({} docs, {} bytes)",
                                    stats.name, stats.document_count, stats.index_size_bytes
                                );
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
            "search" => {
                if parts.len() < 3 {
                    println!("Usage: search <collection> <query>");
                    continue;
                }

                let collection = parts[1];
                let query = parts[2..].join(" ");

                let search_query = SearchQuery {
                    collection: collection.to_string(),
                    query: QueryExpression::FullText {
                        field: "content".to_string(),
                        text: query,
                        boost: None,
                    },
                    limit: Some(5),
                    offset: None,
                    sort: None,
                };

                match engine.search(search_query) {
                    Ok(result) => {
                        println!("Found {} results:", result.total_hits);
                        for (i, hit) in result.documents.iter().enumerate() {
                            println!("{}. {} (score: {:.4})", i + 1, hit.id, hit.score);
                        }
                    }
                    Err(e) => println!("Search error: {}", e),
                }
            }
            "commit" => {
                if parts.len() > 1 {
                    let collection_name = parts[1];
                    match engine.commit_collection(collection_name) {
                        Ok(_) => println!("Committed collection: {}", collection_name),
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    match engine.commit_all().await {
                        Ok(_) => println!("Committed all collections"),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
            "health" => match engine.health_check() {
                Ok(health) => {
                    println!("Engine status: {}", health.status);
                    println!("Collections: {}", health.collections.len());
                }
                Err(e) => println!("Error: {}", e),
            },
            _ => {
                println!(
                    "Unknown command: {}. Type 'help' for available commands.",
                    command
                );
            }
        }
    }

    Ok(())
}
