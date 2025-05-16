use tantivy::TantivyDocument;
use tantivy::schema::OwnedValue;

use crate::schema::SearchFieldId;

//SearchFieldId

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchDocument {
    pub doc: TantivyDocument,
}

impl SearchDocument {
    #[inline(always)]
    pub fn insert(&mut self, SearchFieldId(key): SearchFieldId, value: OwnedValue) {
        self.doc.add_field_value(key, &value)
    }
}

impl From<SearchDocument> for TantivyDocument {
    fn from(value: SearchDocument) -> Self {
        value.doc
    }
}
