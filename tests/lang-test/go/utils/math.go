package utils

// ripex-lang-test: Go math — functions, variadic, generics.
func Add(a, b int) int {
	return a + b
}

func Sum(nums ...int) int {
	total := 0
	for _, n := range nums {
		total += n
	}
	return total
}

func MapInt[T any](xs []T, f func(T) T) []T {
	out := make([]T, len(xs))
	for i, x := range xs {
		out[i] = f(x)
	}
	return out
}

func Squares(xs []int) []int {
	return MapInt(xs, func(x int) int { return x * x })
}
