//+build windows

package main

import (
	"log"
	"os"
	"path/filepath"

	"github.com/itchio/husk/husk"
)

func sample() {
	husk.Hello()

	link, err := husk.NewShellLink()
	must(err)

	target, err := filepath.Abs("./README.md")
	must(err)
	must(link.SetPath(target))

	name := `υπολογιστή.lnk`
	wd, err := os.Getwd()
	must(err)

	absPath := filepath.Join(wd, name)
	log.Printf("Saving to: %s", absPath)

	must(link.Save(absPath))
	link.Free()

	_, err = os.Stat(absPath)
	must(err)

	link, err = husk.NewShellLink()
	must(err)
	must(link.Load(absPath))

	path, err := link.GetPath()
	must(err)

	log.Printf("Shortcut points to: %s", path)
}
