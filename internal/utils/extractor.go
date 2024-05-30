package utils

import (
	"os"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	archiver "github.com/mholt/archiver/v3"
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

/*
Decompress archived files.
*/
func Extract(srcFile, destDir string) (err error) {
	if ok, _ := gutils.PathIsExist(destDir); !ok {
		os.MkdirAll(destDir, os.ModePerm)
	}

	err = archiver.Unarchive(srcFile, destDir)

	if err != nil && strings.HasSuffix(srcFile, ".zip") && runtime.GOOS == gutils.Windows {
		err = UnzipForWindows(srcFile, destDir)
	}
	return
}
