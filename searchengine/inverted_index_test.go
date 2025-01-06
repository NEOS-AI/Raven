package searchengine

import (
	"testing"

	documents "github.com/YeonwooSung/raven/documents"
)

func TestBuildInvertedIndex(t *testing.T) {
	// Create some sample documents
	doc1 := documents.Document{
		ID:      1,
		Content: "This is the first document",
	}
	doc2 := documents.Document{
		ID:      2,
		Content: "This is the second document",
	}
	doc3 := documents.Document{
		ID:      3,
		Content: "This is the third document",
	}

	// Create a slice of documents
	documents := []documents.Document{doc1, doc2, doc3}

	// Call the BuildInvertedIndex function
	invertedIndex := NewPagedInvertedIndex(100, "data/pages")
	invertedIndex.BuildInvertedIndex(documents, false)

	// Perform assertions to verify the correctness of the inverted index

	// Assert that the inverted index contains the expected tokens
	expectedTokens := []string{"this", "is", "the", "first", "document", "second", "third"}
	for _, token := range expectedTokens {
		if _, ok := invertedIndex.CurrentIdx[token]; !ok {
			t.Errorf("Token %s not found in the inverted index", token)
		}
	}

	// Assert that the inverted index contains the expected document IDs for each token
	expectedDocIDs := map[string][]int{
		"this":     {1, 2, 3},
		"is":       {1, 2, 3},
		"the":      {1, 2, 3},
		"first":    {1},
		"document": {1, 2, 3},
		"second":   {2},
		"third":    {3},
	}
	for token, expectedIDs := range expectedDocIDs {
		if ids, ok := invertedIndex.CurrentIdx[token]; ok {
			if !equalSlice(ids, expectedIDs) {
				t.Errorf("Mismatched document IDs for token %s. Expected %v, got %v", token, expectedIDs, ids)
			}
		} else {
			t.Errorf("Token %s not found in the inverted index", token)
		}
	}
}

// Helper function to check if two slices are equal
func equalSlice(a, b []int) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}
