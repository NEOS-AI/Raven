use crate::collection::Collection;
use crate::error::{Result, SearchEngineError};
use crate::types::{
    FieldValue, QueryExpression, SearchHit, SearchQuery, SearchResult, SortField, SortOrder,
};
use std::time::Instant;
use tantivy::schema::Value;
use tantivy::{
    DocAddress, Score, Searcher, TantivyDocument, Term,
    collector::{Count, TopDocs},
    query::*,
    schema::Field,
};

/// Search engine for executing queries against collections
pub struct SearchEngine {
    collection: Collection,
}

impl SearchEngine {
    /// Create a new search engine for a collection
    pub fn new(collection: Collection) -> Self {
        Self { collection }
    }

    /// Execute a search query
    pub fn search(&self, query: SearchQuery) -> Result<SearchResult> {
        let start_time = Instant::now();

        // Get searcher
        let reader = self.collection.index.reader()?;
        let searcher = reader.searcher();

        // Build Tantivy query
        let tantivy_query = self.build_query(&query.query)?;

        // Determine limit and offset
        let limit = query.limit.unwrap_or(10);
        let offset = query.offset.unwrap_or(0);

        // Execute search
        let (top_docs, total_hits) = if offset > 0 {
            // If offset is specified, we need to collect more documents
            let collector = TopDocs::with_limit(offset + limit);
            let top_docs = searcher.search(&tantivy_query, &collector)?;
            let total_collector = Count;
            let total_hits = searcher.search(&tantivy_query, &total_collector)?;

            // Skip documents before offset
            let documents = top_docs.into_iter().skip(offset).collect();
            (documents, total_hits)
        } else {
            let collector = TopDocs::with_limit(limit);
            let top_docs = searcher.search(&tantivy_query, &collector)?;
            let total_collector = Count;
            let total_hits = searcher.search(&tantivy_query, &total_collector)?;
            (top_docs, total_hits)
        };

        // Convert results
        let mut search_hits = Vec::new();
        for (score, doc_address) in top_docs {
            let hit = self.convert_search_hit(&searcher, doc_address, score)?;
            search_hits.push(hit);
        }

        // Apply sorting if specified
        if let Some(sort_fields) = &query.sort {
            self.sort_results(&mut search_hits, sort_fields)?;
        }

        let elapsed = start_time.elapsed();

        Ok(SearchResult {
            total_hits,
            documents: search_hits,
            took_ms: elapsed.as_millis() as u64,
        })
    }

    /// Build Tantivy query from our query expression
    fn build_query(&self, query_expr: &QueryExpression) -> Result<Box<dyn Query>> {
        match query_expr {
            QueryExpression::FullText { field, text, boost } => {
                let field_obj =
                    self.collection
                        .schema_manager
                        .get_field(field)
                        .ok_or_else(|| {
                            SearchEngineError::QueryError(format!("Field '{}' not found", field))
                        })?;

                let mut query: Box<dyn Query> = Box::new(
                    QueryParser::for_index(&self.collection.index, vec![field_obj])
                        .parse_query(text)
                        .map_err(|e| {
                            SearchEngineError::QueryError(format!(
                                "Failed to parse query '{}': {}",
                                text, e
                            ))
                        })?,
                );

                if let Some(boost_value) = boost {
                    query = Box::new(BoostQuery::new(query, *boost_value));
                }

                Ok(query)
            }

            QueryExpression::Term { field, value } => {
                let field_obj =
                    self.collection
                        .schema_manager
                        .get_field(field)
                        .ok_or_else(|| {
                            SearchEngineError::QueryError(format!("Field '{}' not found", field))
                        })?;

                let term = self.build_term(field_obj, value)?;
                Ok(Box::new(TermQuery::new(
                    term,
                    tantivy::schema::IndexRecordOption::Basic,
                )))
            }

            QueryExpression::Range {
                field,
                min,
                max,
                inclusive,
            } => {
                let field_obj =
                    self.collection
                        .schema_manager
                        .get_field(field)
                        .ok_or_else(|| {
                            SearchEngineError::QueryError(format!("Field '{}' not found", field))
                        })?;

                match (min, max) {
                    (Some(FieldValue::I64(min_val)), Some(FieldValue::I64(max_val))) => {
                        // let bound = if *inclusive {
                        //     std::ops::Bound::Included
                        // } else {
                        //     std::ops::Bound::Excluded
                        // };

                        let min_term = Term::from_field_i64(field_obj, *min_val);
                        let max_term = Term::from_field_i64(field_obj, *max_val);
                        let lower_bound = if *inclusive {
                            std::ops::Bound::Included(min_term)
                        } else {
                            std::ops::Bound::Excluded(min_term)
                        };
                        let upper_bound = if *inclusive {
                            std::ops::Bound::Included(max_term)
                        } else {
                            std::ops::Bound::Excluded(max_term)
                        };

                        Ok(Box::new(RangeQuery::new(lower_bound, upper_bound)))
                    }

                    (Some(FieldValue::F64(min_val)), Some(FieldValue::F64(max_val))) => {
                        let min_term = Term::from_field_f64(field_obj, *min_val);
                        let max_term = Term::from_field_f64(field_obj, *max_val);
                        let lower_bound = if *inclusive {
                            std::ops::Bound::Included(min_term)
                        } else {
                            std::ops::Bound::Excluded(min_term)
                        };
                        let upper_bound = if *inclusive {
                            std::ops::Bound::Included(max_term)
                        } else {
                            std::ops::Bound::Excluded(max_term)
                        };

                        Ok(Box::new(RangeQuery::new(lower_bound, upper_bound)))
                    }

                    (Some(FieldValue::Date(min_date)), Some(FieldValue::Date(max_date))) => {
                        let min_dt = tantivy::DateTime::from_timestamp_secs(min_date.timestamp());
                        let max_dt = tantivy::DateTime::from_timestamp_secs(max_date.timestamp());

                        let min_term = Term::from_field_date(field_obj, min_dt);
                        let max_term = Term::from_field_date(field_obj, max_dt);
                        let lower_bound = if *inclusive {
                            std::ops::Bound::Included(min_term)
                        } else {
                            std::ops::Bound::Excluded(min_term)
                        };
                        let upper_bound = if *inclusive {
                            std::ops::Bound::Included(max_term)
                        } else {
                            std::ops::Bound::Excluded(max_term)
                        };

                        Ok(Box::new(RangeQuery::new(lower_bound, upper_bound)))
                    }

                    _ => Err(SearchEngineError::QueryError(
                        "Range query requires min and max values of the same type".to_string(),
                    )),
                }
            }

            QueryExpression::Bool {
                must,
                should,
                must_not,
                minimum_should_match,
            } => {
                let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();

                // Add MUST clauses
                if let Some(must_queries) = must {
                    for query_expr in must_queries {
                        let sub_query = self.build_query(query_expr)?;
                        clauses.push((Occur::Must, sub_query));
                    }
                }

                // Add SHOULD clauses
                if let Some(should_queries) = should {
                    for query_expr in should_queries {
                        let sub_query = self.build_query(query_expr)?;
                        clauses.push((Occur::Should, sub_query));
                    }
                }

                // Add MUST_NOT clauses
                if let Some(must_not_queries) = must_not {
                    for query_expr in must_not_queries {
                        let sub_query = self.build_query(query_expr)?;
                        clauses.push((Occur::MustNot, sub_query));
                    }
                }

                // Create the boolean query
                let bool_query = BooleanQuery::new(clauses);

                // TODO: Handle minimum_should_match when Tantivy supports it

                Ok(Box::new(bool_query))
            }

            QueryExpression::MatchAll => Ok(Box::new(AllQuery)),
        }
    }

    /// Build a Tantivy term from field and value
    fn build_term(&self, field: Field, value: &FieldValue) -> Result<tantivy::Term> {
        let term = match value {
            FieldValue::Text(text) => tantivy::Term::from_field_text(field, text),
            FieldValue::I64(num) => tantivy::Term::from_field_i64(field, *num),
            FieldValue::F64(num) => tantivy::Term::from_field_f64(field, *num),
            FieldValue::Date(date) => {
                let dt = tantivy::DateTime::from_timestamp_secs(date.timestamp());
                tantivy::Term::from_field_date(field, dt)
            }
            FieldValue::Facet(facet_str) => {
                let facet = tantivy::schema::Facet::from_text(facet_str).map_err(|e| {
                    SearchEngineError::QueryError(format!("Invalid facet '{}': {}", facet_str, e))
                })?;
                tantivy::Term::from_field_text(field, &facet.to_string())
            }
            FieldValue::Bytes(_) => {
                return Err(SearchEngineError::QueryError(
                    "Bytes fields are not supported for term queries".to_string(),
                ));
            }
        };

        Ok(term)
    }

    /// Convert Tantivy search result to our format
    fn convert_search_hit(
        &self,
        searcher: &Searcher,
        doc_address: DocAddress,
        score: Score,
    ) -> Result<SearchHit> {
        let doc: TantivyDocument = searcher.doc(doc_address)?;

        // Extract document ID
        let id_field = self
            .collection
            .schema_manager
            .get_field("_id")
            .ok_or_else(|| SearchEngineError::search_error("ID field not found".to_string()))?;

        let id = doc
            .get_first(id_field)
            .and_then(|v| v.to_owned().as_str())
            .ok_or_else(|| SearchEngineError::search_error("Document ID not found".to_string()))?
            .to_string();

        // Convert document fields
        let fields = self.collection.schema_manager.document_from_tantivy(&doc)?;

        Ok(SearchHit { id, score, fields })
    }

    /// Sort search results by specified fields
    fn sort_results(&self, hits: &mut [SearchHit], sort_fields: &[SortField]) -> Result<()> {
        hits.sort_by(|a, b| {
            for sort_field in sort_fields {
                let a_value = a.fields.get(&sort_field.field);
                let b_value = b.fields.get(&sort_field.field);

                let ordering = match (a_value, b_value) {
                    (Some(av), Some(bv)) => self.compare_field_values(av, bv),
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => std::cmp::Ordering::Equal,
                };

                let final_ordering = match sort_field.order {
                    SortOrder::Asc => ordering,
                    SortOrder::Desc => ordering.reverse(),
                };

                if final_ordering != std::cmp::Ordering::Equal {
                    return final_ordering;
                }
            }

            // If all sort fields are equal, sort by score (descending)
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(())
    }

    /// Compare two field values for sorting
    fn compare_field_values(&self, a: &FieldValue, b: &FieldValue) -> std::cmp::Ordering {
        match (a, b) {
            (FieldValue::Text(a), FieldValue::Text(b)) => a.cmp(b),
            (FieldValue::I64(a), FieldValue::I64(b)) => a.cmp(b),
            (FieldValue::F64(a), FieldValue::F64(b)) => {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            }
            (FieldValue::Date(a), FieldValue::Date(b)) => a.cmp(b),
            (FieldValue::Facet(a), FieldValue::Facet(b)) => a.cmp(b),
            (FieldValue::Bytes(a), FieldValue::Bytes(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal, // Different types, consider equal
        }
    }
}

// Custom error for search-specific issues
impl SearchEngineError {
    pub fn search_error(msg: impl Into<String>) -> Self {
        SearchEngineError::CustomError(format!("Search error: {}", msg.into()))
    }
}
