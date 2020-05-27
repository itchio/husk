package husk

import (
	"fmt"

	"github.com/itchio/husk/lowhusk"
)

func AsString(xs *lowhusk.XString) string {
	s := lowhusk.XstringData(xs)[:lowhusk.XstringLen(xs)]
	lowhusk.XstringFree(xs)
	return s
}

func AsError(err *lowhusk.XString) error {
	s := AsString(err)
	lowhusk.XstringFree(err)
	return &HuskError{s}
}

type HuskError struct {
	Message string
}

func (he *HuskError) Error() string {
	return fmt.Sprintf("husk error: %s", he.Message)
}

type ShellLink struct {
	inner *lowhusk.ShellLink
}

func NewShellLink() (*ShellLink, error) {
	var err *lowhusk.XString
	var link *lowhusk.ShellLink
	if lowhusk.ShellLinkNew(&link, &err) != 0 {
		return nil, AsError(err)
	}
	return &ShellLink{link}, nil
}

func (l *ShellLink) Load(path string) error {
	var pathBytes = []byte(path)
	var err *lowhusk.XString
	if lowhusk.ShellLinkLoad(l.inner, &pathBytes[0], uint64(len(pathBytes)), &err) != 0 {
		return AsError(err)
	}
	return nil
}

func (l *ShellLink) GetPath() (string, error) {
	var path *lowhusk.XString
	var err *lowhusk.XString
	if lowhusk.ShellLinkGetPath(l.inner, &path, &err) != 0 {
		return "", AsError(err)
	}
	return AsString(path), nil
}

func (l *ShellLink) Free() {
	lowhusk.ShellLinkFree(l.inner)
}
