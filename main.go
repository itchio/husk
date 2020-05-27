package main

import (
	"fmt"
	"log"

	"github.com/itchio/husk/husk"
)

func main() {
	link, err := husk.NewShellLink()
	must(err)

	must(link.Load("C:\\Users\\amos\\Desktop\\υπολογιστή-does-not-exist.lnk"))

	path, err := link.GetPath()
	must(err)

	log.Printf("path = %s", path)
}

func must(err error) {
	if err != nil {
		panic(fmt.Sprintf("%+v", err))
	}
}
