//+build windows

package main

import (
	"log"

	"github.com/itchio/husk/husk"
)

func sample() {
	husk.Hello()

	link, err := husk.NewShellLink()
	must(err)

	must(link.Load(`C:\Windows\WinSxS\amd64_microsoft-windows-wordpad_31bf3856ad364e35_10.0.18362.267_none_837ed916fe8dbd2f\Wordpad.lnk`))

	path, err := link.GetPath()
	must(err)

	log.Printf("path = %s", path)
}
