package utils

import (
	"os"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
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
	err := os.Symlink(oldname, newname)
	if runtime.GOOS == gutils.Windows {
		// Hardlink for windows files. Hardlink for windows in the same disk partion is supported.
		// Softlink for windows files need admin previllege.
		if err != nil {
			err = os.Link(oldname, newname)
		}
	}
	if err != nil {
		gprint.PrintError("create symbolic failed: %+v\n", err)
		if runtime.GOOS == gutils.Windows {
			gprint.PrintWarning("If you're on Windows11, then go to 'System>For developers>' and enable the 'Developer Mode'.")
			gprint.PrintWarning("If you're on Windows10, then you can use vm with 'Admin Privilege'.")
			gprint.PrintInfo("Note that, FAT32 and exFAT do not support symbolics. So you should not use vm with these partitions.")
		}
		os.Exit(1)
	}
	return err
}
