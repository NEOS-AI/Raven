package routes

import (
	"github.com/gofiber/fiber/v2"

	searchengine "raven/searchengine"
)

var searchEngine *searchengine.SearchEngine

func SearchRoute(app *fiber.App, search_engine *searchengine.SearchEngine) {
	searchEngine = search_engine
	app.Get("/search", func(c *fiber.Ctx) error {
		//TODO search with search engine
		return c.SendString("Hello, World 👋!")
	})
}
