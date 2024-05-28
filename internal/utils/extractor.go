package utils

import (
	"os"

	"github.com/gvcgo/goutils/pkgs/gutils"
	archiver "github.com/mholt/archiver/v3"
)

/*
Decompress archived files.
*/
func Extract(srcFile, destDir string) (err error) {
	if ok, _ := gutils.PathIsExist(destDir); !ok {
		os.MkdirAll(destDir, os.ModePerm)
	}
	return archiver.Unarchive(srcFile, destDir)
}
