package main

// ripex-lang-test: Go entry — cross-package imports, call graph.
import (
	"fmt"

	"ripex-lang-test/go/models"
	"ripex-lang-test/go/utils"
	"ripex-lang-test/go/services"
)

func main() {
	alice := models.NewUser("Alice", "alice@example.com")
	alice.Roles = []string{"admin"}
	fmt.Println(utils.Greet(alice.Name))
	fmt.Println(alice.Describe())

	widget := models.Product{ID: 1, Name: "Widget", Price: 19.99, Cat: models.Electronics}
	fmt.Println(widget.PriceOf())

	results := services.FetchAll([]string{"a", "b"})
	fmt.Println(utils.FilterPositive([]int{-1, 2, 3}))
	fmt.Println(utils.Sum(utils.Squares([]int{1, 2, 3})...))
	_ = results
}
