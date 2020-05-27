package husk

import (
	"fmt"
	"reflect"
	"unsafe"

	"github.com/itchio/husk/lowhusk"
)

func AsString(xs *lowhusk.XString) string {
	// this returns `*const u8`, ie. `*byte` in cgo, we need 2 casts
	xdata := uintptr(unsafe.Pointer(lowhusk.XstringData(xs)))
	// this returns `usize`, ie. `uintptr_t` in cgo, we need 1 cast
	xlen := int(lowhusk.XstringLen(xs))
	// this builds a slice that refers to the data in `xs`
	// n.b: "Cap" is irrelevant, we never mutate it, got forbid
	sh := &reflect.SliceHeader{
		Data: xdata,
		Len:  xlen,
		Cap:  xlen,
	}
	// this slice will become invalid as soon as `XstringFree` is called
	slice := *(*[]byte)(unsafe.Pointer(sh))
	// this copies out from the slice (not obvious)
	s := string(slice)
	// ...so now `xs` can be freed
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
