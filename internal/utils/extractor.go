package utils

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"

	Arch "github.com/gvcgo/goutils/pkgs/archiver"
)

func UnzipForWindows(zipFilePath, dstDir string) error {
	// expand -r file.zip C:\Users\username\Desktop\extracted
	_, err := gutils.ExecuteSysCommand(true, "",
		"powershell",
		"expand",
		"-r",
		zipFilePath,
		dstDir)
	return err
}

// use archiver or xtract.
func UseArchiver(srcPath string) bool {
	if strings.HasSuffix(srcPath, ".gz") && !strings.HasSuffix(srcPath, ".tar.gz") {
		return false
	}
	if strings.HasSuffix(srcPath, ".7z") {
		return false
	}
	if strings.Contains(strings.ToLower(srcPath), "odin") {
		return false
	}
	return true
}

/*
Decompress archived files.
*/
func Extract(srcFile, destDir string) (err error) {
	if ok, _ := gutils.PathIsExist(destDir); !ok {
		os.MkdirAll(destDir, os.ModePerm)
	}

	// err = archiver.Unarchive(srcFile, destDir)
	if arch, err1 := Arch.NewArchiver(srcFile, destDir, UseArchiver(srcFile)); err1 == nil {
		_, err = arch.UnArchive()
		if err != nil && runtime.GOOS == gutils.Windows && strings.HasSuffix(srcFile, ".zip") {
			err = UnzipForWindows(srcFile, destDir)
		}

		// for odin.
		tempDirList, _ := os.ReadDir(destDir)
		for _, d := range tempDirList {
			dd := filepath.Join(destDir, d.Name())
			if !d.IsDir() && strings.HasSuffix(d.Name(), ".zip") {
				aa, _ := Arch.NewArchiver(dd, destDir, true)
				aa.UnArchive()
			}
		}
		return
	} else {
		err = err1
	}
	return
}
