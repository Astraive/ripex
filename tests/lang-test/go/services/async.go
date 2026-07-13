package services

import (
	"context"
	"fmt"
	"time"
)

// ripex-lang-test: Go concurrency — goroutines, channels, context.
func FetchAll(urls []string) []string {
	ch := make(chan string, len(urls))
	for _, u := range urls {
		go func(url string) {
			ch <- url
		}(u)
	}
	out := make([]string, 0, len(urls))
	for range urls {
		out = append(out, <-ch)
	}
	return out
}

func WithTimeout(ctx context.Context) string {
	select {
	case <-time.After(time.Second):
		return "slow"
	case <-ctx.Done():
		return "cancelled"
	}
}

func Log(msg string) {
	fmt.Println(msg)
}
