package models

// ripex-lang-test: Go Product model — interface, embedded struct.
type Category string

const (
	Electronics Category = "electronics"
	Clothing    Category = "clothing"
)

type Product struct {
	ID     int
	Name   string
	Price  float64
	Cat    Category
}

type Pricable interface {
	PriceOf() float64
}

func (p Product) PriceOf() float64 {
	return p.Price
}
