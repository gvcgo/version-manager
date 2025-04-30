package vui

import (
	"os"

	"golang.org/x/term"
)

func GetTermSize() (height, width int, err error) {
	fd := os.Stdout.Fd()
	return term.GetSize(int(fd))
}
