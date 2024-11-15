package searchengine

import (
	documents "raven/documents"
	nlp "raven/nlp"
	bloomfilter "raven/searchengine/bloomfilter"
	"strings"
)

type InvertedIndex map[string][]int

func UpdateInvertedIndexWithDoc(index InvertedIndex, doc documents.Document, useTokenizer bool, sbf *bloomfilter.ScalableBloomFilter) {
	// support both pre-trained sentence-piece tokenizer and simple whitespace tokenizer
	var tokens []string
	if useTokenizer {
		tokens = nlp.Tokenize_Query(strings.ToLower(doc.Content))
	} else {
		tokens = strings.Fields(strings.ToLower(doc.Content))
	}

	// iterate all tokens in the document, and store the document ID to the key-value store
	for _, token := range tokens {
		if _, ok := index[token]; !ok {
			index[token] = make([]int, 0)
		}
		index[token] = append(index[token], doc.ID)

		// add the token to the Bloom filter
		sbf.Add([]byte(token))
	}
}

/**
 * Build an inverted index by tokenizing the documents and storing the doucment IDs to key-value store.
 * The key is the token, and the value is a slice of document IDs.
 *
 * @param documents A slice of documents
 * @return InvertedIndex
 */
func BuildInvertedIndex(documents []documents.Document, useTokenizer bool) (InvertedIndex, *bloomfilter.ScalableBloomFilter) {
	index := make(InvertedIndex)
	sbf, _ := bloomfilter.NewScalable(bloomfilter.ParamsScalable{InitialSize: 1000, FalsePositiveRate: 0.01, FalsePositiveGrowth: 2})

	// iterate all documents
	for _, doc := range documents {
		UpdateInvertedIndexWithDoc(index, doc, useTokenizer, sbf)
	}

	return index, sbf
}
