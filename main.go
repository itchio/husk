package main

import (
	"fmt"
)

func main() {
	sample()
}

func must(err error) {
	if err != nil {
		panic(fmt.Sprintf("%+v", err))
	}
}
