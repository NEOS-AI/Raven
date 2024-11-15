# Raven

Raven is a cloud-native search engine database.

## Implemented Features

* Indexing
    * Inverted Index
* Search
    * TF-IDF
    * BM25
* Natural Language Processing
    * Subword Tokenization
    * Stopword Removal
    * Language Detection

## Profiling and Tracing

Basically, this application uses `net/http/pprof` for profiling and tracing.

For visualizing the profiling and tracing, open `http://localhost:6060/debug/pprof/` in your browser.

## ToDos

* [v] Use bloomfilter for filtering the UNK tokens
* [ ] Build index from reading and parsing raw text files
* [ ] Save and load index and bloom filters to file
* [ ] Build fuzzy full-text search by using SuffixTree
* [ ] Levenshtein Distance Spell Correction
* [ ] Pseudo Relevance Feedback
* [ ] Query Expansion
* [ ] Add support for vector index
    * [ ] HNSW
    * [ ] Flat

## References

- [blurfx/mini-search-engine](https://github.com/blurfx/mini-search-engine)
- [System Design for Discovery](https://eugeneyan.com/writing/system-design-for-discovery/)
- [🤗 bert-base-multilingual-cased](https://huggingface.co/bert-base-multilingual-cased)
- [sugarme/tokenizer](https://pkg.go.dev/github.com/sugarme/tokenizer)
