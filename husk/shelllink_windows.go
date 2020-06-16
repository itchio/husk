//+build windows

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
	// n.b: "Cap" is irrelevant, we never mutate it.
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
	return &HuskError{AsString(err)}
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

func (l *ShellLink) Save(path string) error {
	var pathBytes = []byte(path)
	var err *lowhusk.XString
	if lowhusk.ShellLinkSave(l.inner, &pathBytes[0], uint64(len(pathBytes)), &err) != 0 {
		return AsError(err)
	}
	return nil
}

func (l *ShellLink) SetPath(path string) error {
	var pathBytes = []byte(path)
	var err *lowhusk.XString
	if lowhusk.ShellLinkSetPath(l.inner, &pathBytes[0], uint64(len(pathBytes)), &err) != 0 {
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

func (l *ShellLink) SetArguments(arguments string) error {
	var pathBytes = []byte(arguments)
	var err *lowhusk.XString
	if lowhusk.ShellLinkSetArguments(l.inner, &pathBytes[0], uint64(len(pathBytes)), &err) != 0 {
		return AsError(err)
	}
	return nil
}

func (l *ShellLink) GetArguments() (string, error) {
	var arguments *lowhusk.XString
	var err *lowhusk.XString
	if lowhusk.ShellLinkGetArguments(l.inner, &arguments, &err) != 0 {
		return "", AsError(err)
	}
	return AsString(arguments), nil
}

func (l *ShellLink) SetDescription(description string) error {
	var pathBytes = []byte(description)
	var err *lowhusk.XString
	if lowhusk.ShellLinkSetDescription(l.inner, &pathBytes[0], uint64(len(pathBytes)), &err) != 0 {
		return AsError(err)
	}
	return nil
}

func (l *ShellLink) GetDescription() (string, error) {
	var description *lowhusk.XString
	var err *lowhusk.XString
	if lowhusk.ShellLinkGetDescription(l.inner, &description, &err) != 0 {
		return "", AsError(err)
	}
	return AsString(description), nil
}

func (l *ShellLink) SetWorkingDirectory(workingDirectory string) error {
	var pathBytes = []byte(workingDirectory)
	var err *lowhusk.XString
	if lowhusk.ShellLinkSetWorkingDirectory(l.inner, &pathBytes[0], uint64(len(pathBytes)), &err) != 0 {
		return AsError(err)
	}
	return nil
}

func (l *ShellLink) GetWorkingDirectory() (string, error) {
	var workingDirectory *lowhusk.XString
	var err *lowhusk.XString
	if lowhusk.ShellLinkGetWorkingDirectory(l.inner, &workingDirectory, &err) != 0 {
		return "", AsError(err)
	}
	return AsString(workingDirectory), nil
}

func (l *ShellLink) SetIconLocation(iconLocation string, iconIndex int) error {
	var bs = []byte(iconLocation)
	var err *lowhusk.XString
	if lowhusk.ShellLinkSetIconLocation(l.inner, &bs[0], uint64(len(bs)), iconIndex, &err) != 0 {
		return AsError(err)
	}
	return nil
}

func (l *ShellLink) GetIconLocation() (string, int, error) {
	var iconLocation *lowhusk.XString
	var iconIndex int
	var err *lowhusk.XString
	if lowhusk.ShellLinkGetIconLocation(l.inner, &iconLocation, &iconIndex, &err) != 0 {
		return "", 0, AsError(err)
	}
	return AsString(iconLocation), int(iconIndex), nil
}

func (l *ShellLink) Free() {
	lowhusk.ShellLinkFree(l.inner)
}
