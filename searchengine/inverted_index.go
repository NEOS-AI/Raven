package searchengine

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	documents "github.com/YeonwooSung/raven/documents"
	nlp "github.com/YeonwooSung/raven/nlp"
	bloomfilter "github.com/YeonwooSung/raven/searchengine/bloomfilter"
)

type InvertedIndex map[string][]int

type Page struct {
	ID    int           `json:"id"`
	Index InvertedIndex `json:"index"`
}

type PagedInvertedIndex struct {
	PageSize    int                              // Max tokens per page
	PagesDir    string                           // directory to store pages
	CurrentID   int                              // Current page ID
	CurrentIdx  InvertedIndex                    // Current in-memory index
	BloomFilter *bloomfilter.ScalableBloomFilter // Bloom filter
}

// NewPagedInvertedIndex: Create a new paged inverted index
//
// input:
//
//	pageSize: The maximum number of tokens per page
//	pagesDir: The directory to store the pages
//
// return: A pointer to the new paged inverted index
func NewPagedInvertedIndex(pageSize int, pagesDir string) *PagedInvertedIndex {
	if _, err := os.Stat(pagesDir); os.IsNotExist(err) {
		os.MkdirAll(pagesDir, 0755)
	}

	sbf, _ := bloomfilter.NewScalable(bloomfilter.ParamsScalable{
		InitialSize:         1000,
		FalsePositiveRate:   0.01,
		FalsePositiveGrowth: 2,
	})

	return &PagedInvertedIndex{
		PageSize:    pageSize,
		PagesDir:    pagesDir,
		CurrentID:   0,
		CurrentIdx:  make(InvertedIndex),
		BloomFilter: sbf,
	}
}

// UpdateInvertedIndexWithDoc: Update the inverted index with a document
//
// input:
//
//	doc: A document
//	useTokenizer: A boolean value to determine whether to use tokenizer
//
// return: None
func (pii *PagedInvertedIndex) UpdateInvertedIndexWithDoc(doc documents.Document, useTokenizer bool) {
	var tokens []string
	if useTokenizer {
		tokens = nlp.Tokenize_Query(strings.ToLower(doc.Content))
	} else {
		tokens = strings.Fields(strings.ToLower(doc.Content))
	}

	for _, token := range tokens {
		if _, ok := pii.CurrentIdx[token]; !ok {
			pii.CurrentIdx[token] = make([]int, 0)
		}
		pii.CurrentIdx[token] = append(pii.CurrentIdx[token], doc.ID)

		// Bloom filter 업데이트
		pii.BloomFilter.Add([]byte(token))

		// 페이지 크기 초과 시 디스크에 저장
		if len(pii.CurrentIdx) >= pii.PageSize {
			fmt.Printf("len(pii.CurrentIdx) = %d, pii.PageSize = %d\n", len(pii.CurrentIdx), pii.PageSize)
			pii.FlushToDisk()
		}
	}
}

// FlushToDisk: Flush the current in-memory index to disk
//
// input: None
// return: None
func (pii *PagedInvertedIndex) FlushToDisk() {
	page := Page{
		ID:    pii.CurrentID,
		Index: pii.CurrentIdx,
	}

	data, err := json.Marshal(page)
	if err != nil {
		fmt.Println("Error marshalling page:", err)
		return
	}

	filename := filepath.Join(pii.PagesDir, fmt.Sprintf("page_%d.json", pii.CurrentID))
	err = os.WriteFile(filename, data, 0644) // os.WriteFile 사용
	if err != nil {
		fmt.Println("Error writing page to disk:", err)
		return
	}

	fmt.Printf("Flushed page %d to disk\n", pii.CurrentID)
	pii.CurrentID++
	pii.CurrentIdx = make(InvertedIndex) // 현재 인덱스 초기화
}

// Search: Search for a term in the inverted index
//
// input:
//
//	term: A string term to search for
//
// return: A slice of document IDs containing the term
func (pii *PagedInvertedIndex) Search(term string) []int {
	results := []int{}

	// Bloom filter로 빠른 존재 여부 확인
	contains, err := pii.BloomFilter.Test([]byte(term))
	if err != nil {
		fmt.Println("Error testing bloom filter:", err)
		return results
	}
	if !contains {
		fmt.Println("Term not found in bloom filter: ", term)
		return results // Bloom 필터에 없는 경우 즉시 반환
	}

	files, err := os.ReadDir(pii.PagesDir) // os.ReadDir 사용
	if err != nil {
		fmt.Println("Error reading pages directory:", err)
		return results
	}

	for _, file := range files {
		filename := filepath.Join(pii.PagesDir, file.Name())
		data, err := os.ReadFile(filename) // os.ReadFile 사용
		if err != nil {
			fmt.Println("Error reading page file:", err)
			continue
		}

		var page Page
		err = json.Unmarshal(data, &page)
		if err != nil {
			fmt.Println("Error unmarshalling page:", err)
			continue
		}

		if docIDs, ok := page.Index[term]; ok {
			results = append(results, docIDs...)
		}
	}

	// remove duplicates
	results = removeDuplicates(results)

	return results
}

// BuildInvertedIndex: Build inverted index from documents
//
// input:
//
//	docs: A slice of documents
//	useTokenizer: A boolean value to determine whether to use tokenizer
//
// return: None
func (pii *PagedInvertedIndex) BuildInvertedIndex(docs []documents.Document, useTokenizer bool) {
	for _, doc := range docs {
		pii.UpdateInvertedIndexWithDoc(doc, useTokenizer)
	}
}

// removeDuplicates: Remove duplicates from a slice
//
// input: A slice of integers
// return: A slice of integers without duplicates
func removeDuplicates(input []int) []int {
	uniqueMap := make(map[int]bool)
	uniqueSlice := []int{}

	for _, item := range input {
		if !uniqueMap[item] {
			uniqueMap[item] = true
			uniqueSlice = append(uniqueSlice, item)
		}
	}

	return uniqueSlice
}
