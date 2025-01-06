package searchengine

// Dump: Dump the search engine to disk
func (engine SearchEngine) Dump() {
	engine.Index.FlushToDisk()
}
