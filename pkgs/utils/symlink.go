package utils

import (
	"os"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

func IsDir(path string) bool {
	s, err := os.Stat(path)
	if err != nil {
		return false
	}
	return s.IsDir()
}

func IsFile(path string) bool {
	return !IsDir(path)
}

func SymbolicLink(oldname, newname string) error {
	if runtime.GOOS == gutils.Windows && IsFile(oldname) {
		// Hardlink for windows files. Hardlink for windows in the same disk partion is supported.
		// Softlink for windows files need admin previllege.
		return os.Link(oldname, newname)
	} else {
		return os.Symlink(oldname, newname)
	}
}
