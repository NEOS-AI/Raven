package main

import (
	"bufio"
	"fmt"
	"math"

	"net/http"
	_ "net/http/pprof"

	"os"
	"strings"

	logger "github.com/YeonwooSung/raven/cluster/log"
	documents "github.com/YeonwooSung/raven/documents"
	nlp "github.com/YeonwooSung/raven/nlp"
	searchengine "github.com/YeonwooSung/raven/searchengine"

	"github.com/gofiber/fiber"
	"github.com/gofiber/fiber/v2/middleware/cors"
)

var SearchEngine searchengine.SearchEngine

func init() {
	// init the singleton logger
	logger.RavenLogger.InitLoggers()

	// docs := ParseFetchAndReturnDocuments()
	docs := []documents.Document{
		{ID: 0, Content: "Lorem ipsum blah blah fox"},
		{ID: 1, Content: "The quick brown fox jumped over the lazy dog. The dog slept peacefully."},
		{ID: 2, Content: "I have a dream that one day this nation will rise up and live out the true meaning of its creed: 'We hold these truths to be self-evident, that all men are created equal.'"},
		{ID: 3, Content: "To be, or not to be, that is the question: Whether 'tis nobler in the mind to suffer The slings and arrows of outrageous fortune, Or to take arms against a sea of troubles And by opposing end them."},
		{ID: 4, Content: "In a hole in the ground there lived a hobbit. Not a nasty, dirty, wet hole, filled with the ends of worms and an oozy smell, nor yet a dry, bare, sandy hole with nothing in it to sit down on or to eat: it was a hobbit-hole, and that means comfort."},
		{ID: 5, Content: "The only way to do great work is to love what you do. If you haven't found it yet, keep looking. Don't settle. As with all matters of the heart, you'll know when you find it."},
		{ID: 6, Content: "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife."},
		{ID: 7, Content: "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, it was the season of Darkness, it was the spring of hope, it was the winter of despair."},
		{ID: 8, Content: "Two households, both alike in dignity, In fair Verona, where we lay our scene, From ancient grudge break to new mutiny, Where civil blood makes civil hands unclean."},
		{ID: 9, Content: "Once upon a time in a far-off land, there was a princess who was very beautiful and very kind, but also very sad."},
		{ID: 10, Content: "It is not in the stars to hold our destiny but in ourselves."},
		{ID: 11, Content: "In the beginning God created the heaven and the earth. And the earth was without form, and void; and darkness was upon the face of the deep. And the Spirit of God moved upon the face of the waters."},
		{ID: 12, Content: "There are known knowns; there are things we know we know. We also know there are known unknowns; that is to say we know there are some things we do not know. But there are also unknown unknowns – the ones we don't know we don't know."},
		{ID: 13, Content: "When I consider how my light is spent Ere half my days in this dark world and wide, And that one talent which is death to hide Lodg'd with me useless, though my soul more bent To serve therewith my Maker, and present My true account, lest he returning chide;"},
		{ID: 14, Content: "I wandered lonely as a cloud That floats on high o'er vales and hills, When all at once I saw a crowd, A host, of golden daffodils; Beside the lake, beneath the trees, Fluttering and dancing in the breeze."},
		{ID: 15, Content: "Do not go gentle into that good night, Old age should burn and rave at close of day; Rage, rage against the dying of the light."},
		{ID: 16, Content: "The sun was shining on the sea, Shining with all his might: He did his very best to make The billows smooth and bright."},
		{ID: 17, Content: "In Xanadu did Kubla Khan A stately pleasure-dome decree: Where Alph, the sacred river, ran Through caverns measureless to man Down to a sunless sea."},
		{ID: 18, Content: "I celebrate myself, and sing myself, And what I assume you shall assume, For every atom belonging to me as good belongs to you."},
		{ID: 19, Content: "The love that moves the sun and all the stars."},
		{ID: 20, Content: "It was a bright cold day in April, and the clocks were striking thirteen. Winston Smith, his chin nuzzled into his breast in an effort to escape the vile wind, slipped quickly through the glass doors of Victory Mansions, though not quickly enough to prevent a swirl of gritty dust from entering along with him."},
		{ID: 21, Content: "It was a pleasure to burn. It was a special pleasure to see things eaten, to see things blackened and changed."},
		{ID: 22, Content: "The human race, to which so many of my readers belong, has been playing at children's games from the beginning, and will probably do it till the end, which is a nuisance for the few people who grow up. And one of the games to which it is most attached is called 'Keep to-morrow dark,' and which is also sometimes called 'Cheat the Prophet.'"},
		{ID: 23, Content: "Happy families are all alike; every unhappy family is unhappy in its own way."},
		{ID: 24, Content: "I am an invisible man. No, I am not a spook like those who haunted Edgar Allan Poe; nor am I one of your Hollywood-movie ectoplasms. I am a man of substance, of flesh and bone, fiber and liquids—and I might even be said to possess a mind. I am invisible, understand, simply because people refuse to see me."},
		{ID: 25, Content: "It was a dark and stormy night; the rain fell in torrents, except at occasional intervals, when it was checked by a violent gust of wind which swept up the streets (for it is in London that our scene lies), rattling along the housetops, and fiercely agitating the scanty flame of the lamps that struggled against the darkness."},
		{ID: 26, Content: "The sky above the port was the color of television, tuned to a dead channel."},
		{ID: 27, Content: "All children, except one, grow up. They soon know that they will grow up, and the way Wendy knew was this. One day when she was two years old she was playing in a garden, and she plucked another flower and ran with it to her mother. I suppose she must have looked rather delightful, for Mrs. Darling put her hand to her heart and cried, 'Oh, why can't you remain like this for ever!' This was all that passed between them on the subject, but henceforth Wendy knew that she must grow up. You always know after you are two. Two is the beginning of the end."},
		{ID: 28, Content: "As Gregor Samsa awoke one morning from uneasy dreams he found himself transformed in his bed into a gigantic insect."},
		{ID: 29, Content: "Call me Ishmael. Some years ago—never mind how long precisely—having little or no money in my purse, and nothing particular to interest me on shore, I thought I would sail about a little and see the watery part of the world."},
		{ID: 30, Content: "It was the day my grandmother exploded."},
		{ID: 31, Content: "아이유(IU, 본명: 이지은, 李知恩, 1993년 5월 16일~)는 대한민국의 가수이자 배우이다. 배우로 활동할 때도 예명을 사용한다. '아이유(IU)'라는 예명은 'I'와 'You'를 합친 합성어로 '너와 내가 음악으로 하나가 된다'라는 의미이다"},
	}
	SaveDocsAsCsv(docs, "data/result.csv")

	// initialize the tokenizer
	nlp.Init_Tokenizer()

	useTokenizer := true
	index, sbf := searchengine.BuildInvertedIndex(docs, useTokenizer)
	docLength := 0.
	count := 0
	maxThreshold := math.MaxFloat64 - 100
	for _, doc := range docs {
		currentDocLength := float64(len(doc.Content))
		// check if docLength + currentDocLength is too large to avoid overflow
		if docLength+currentDocLength > maxThreshold {
			break
		}

		// increase the docLength
		docLength += currentDocLength
		count++
	}
	// to avoid division by zero
	if count == 0 {
		panic("No documents to index")
	}
	countF := float64(count)

	// initialize the search engine
	SearchEngine = searchengine.SearchEngine{
		Index:         index,
		Documents:     docs,
		TotalDocCount: countF,
		TotalDocLen:   docLength,
		AvgDocLength:  docLength / countF,
		K1:            1.2,
		B:             0.75,
		Bloomfilter:   sbf,
	}
}

func main() {
	// start the pprof server
	go func() {
		fmt.Println(http.ListenAndServe("0.0.0.0:6060", nil))
	}()

	app := fiber.New()
	app.Use(cors.New())

	// run endless loop to accept search queries from the user
	for {
		fmt.Print("Enter a search query: ")
		query, _ := bufio.NewReader(os.Stdin).ReadString('\n')
		query = strings.TrimSpace(query)
		if query == "" {
			continue
		}
		results := SearchEngine.Search(query, 20)
		fmt.Printf("%d results for query '%s':\n", len(results), query)
		for _, result := range results {
			fmt.Printf("- %s (score=%.2f)\n", result.Content, result.Score)
		}
	}
}
