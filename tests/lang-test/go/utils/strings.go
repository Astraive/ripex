package utils

import "strings"

// ripex-lang-test: Go string utils — closures, slices.
func Greet(name string) string {
	return "Hello, " + name + "!"
}

func MaskEmail(email string) string {
	parts := strings.Split(email, "@")
	return parts[0][:1] + "***@" + parts[1]
}

func FilterPositive(xs []int) []int {
	var out []int
	for _, x := range xs {
		if x > 0 {
			out = append(out, x)
		}
	}
	return out
}
