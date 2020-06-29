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

	target, err := filepath.Abs("./husk-sample.exe")
	must(err)
	must(link.SetPath(target))
	must(link.SetArguments("--level high"))
	must(link.SetDescription("C'est tous les jours noël"))
	must(link.SetWorkingDirectory(`C:\`))
	must(link.SetIconLocation(`C:\Windows\System32\notepad.exe`, 0))

	name := `υπολογιστή.lnk`
	wd, err := os.Getwd()
	must(err)

	absPath := filepath.Join(wd, name)
	log.Printf("Saving to: %s", absPath)

	done := make(chan struct{})
	go func() {
		must(link.Save(absPath))
		done <- struct{}{}
	}()
	<-done
	link.Free()

	_, err = os.Stat(absPath)
	must(err)

	link, err = husk.NewShellLink()
	must(err)
	must(link.Load(absPath))

	path, err := link.GetPath()
	must(err)
	log.Printf("Shortcut points to: %s", path)

	arguments, err := link.GetArguments()
	must(err)
	log.Printf("Shortcut has arguments: %s", arguments)

	description, err := link.GetDescription()
	must(err)
	log.Printf("Shortcut has description: %s", description)

	workingDirectory, err := link.GetWorkingDirectory()
	must(err)
	log.Printf("Shortcut has working directory: %s", workingDirectory)

	iconLocation, iconIndex, err := link.GetIconLocation()
	must(err)
	log.Printf("Shortcut has icon location: %s, index %d", iconLocation, iconIndex)
}
