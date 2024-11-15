package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	documents "raven/documents"
)

func parseFetchAndReturnDocument(record []string) documents.Document {
	url := record[0]
	goodOrBad := record[1]
	if goodOrBad != "good" {
		return documents.Document{}
	}

	content := OnPage(url)
	// if content is empty, return an empty document
	if content == "" {
		return documents.Document{}
	}

	// create a new document
	doc := documents.Document{
		Content: content,
		Url:     url,
	}
	return doc
}

func ParseFetchAndReturnDocuments() []documents.Document {
	records := ReadCsvFile("data/urldata.csv")

	documents := make([]documents.Document, 0)
	for i, record := range records {
		// print out every 1000 rows
		if i%100 == 0 {
			log.Println("Processing row", i)
		}

		// skip the header row
		if i == 0 {
			continue
		}
		document := parseFetchAndReturnDocument(record)
		if document.Content == "" {
			continue
		}
		documents = append(documents, document)
	}
	return documents
}

func ReadCsvFile(filePath string) [][]string {
	f, err := os.Open(filePath)
	if err != nil {
		log.Fatal("Unable to read input file "+filePath, err)
	}
	defer f.Close()

	csvReader := csv.NewReader(f)

	records := make([][]string, 0)
	for {
		record, err := csvReader.Read()
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Println("Error reading the record", err)
			continue
		}
		records = append(records, record)
	}

	return records
}

func OnPage(link string) string {
	client := http.Client{
		Timeout: 3 * time.Second,
	}
	// if link does not start with http, add it
	if !strings.HasPrefix(link, "http") {
		link = "https://" + link
	}
	// replace the space with ""
	link = strings.ReplaceAll(link, " ", "")

	res, err := client.Get(link)
	if err != nil {
		// print out the error msg, and return an empty string
		log.Println("Error fetching the URL", err)
		return ""
	}

	// read the response body
	content, err := io.ReadAll(res.Body)
	res.Body.Close()
	if err != nil {
		// log.Fatal(err)
		log.Println("Error reading response body", err)
		return ""
	}
	return string(content)
}

func SaveDocsAsCsv(docs []documents.Document, outputFilePath string) {
	// create a new file
	file, err := os.Create(outputFilePath)
	if err != nil {
		log.Fatal("Cannot create file", err)
	}
	defer file.Close()

	// create a new CSV writer
	writer := csv.NewWriter(file)
	defer writer.Flush()

	// write the header
	writer.Write([]string{"ID", "Content", "URL"})
	for i, doc := range docs {
		writer.Write([]string{fmt.Sprint(i), doc.Content, doc.Url})
	}

	if err := writer.Error(); err != nil {
		log.Fatal("Cannot write to file", err)
	}
}
