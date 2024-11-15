package searchengine

import (
	"github.com/bbalet/stopwords"

	nlp "raven/nlp"
)

func removeStopwords(query string) string {
	language, isExist := nlp.DetectLanguage(query)
	if !isExist {
		return query
	}

	cleaned_query := stopwords.CleanString(query, language, true)
	if cleaned_query == "" {
		return query
	}
	return cleaned_query
}
