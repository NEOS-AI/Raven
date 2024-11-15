package searchengine

import (
	"github.com/bbalet/stopwords"

	"reflect"
	"testing"
)

func TestCleanStopwords(t *testing.T) {
	// Define the input and expected output
	input := "This is a test sentence with some stopwords."

	// Call the function to be tested
	expectedOutput := stopwords.CleanString(input, "english", true)

	// Call the function to be tested
	actualOutput := removeStopwords(input)

	// Compare the actual output with the expected output
	if !reflect.DeepEqual(actualOutput, expectedOutput) {
		t.Errorf("Expected %s, but got %s", expectedOutput, actualOutput)
	}
}
