package utils

import (
	"os"
	"path/filepath"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

func ClearEmptyDirs(dPath string) {
	if ok, _ := gutils.PathIsExist(dPath); !ok {
		return
	}

	if ok := IsDir(dPath); !ok {
		return
	}
	dList, _ := os.ReadDir(dPath)
	for _, d := range dList {
		if d.IsDir() {
			dirPath := filepath.Join(dPath, d.Name())
			l, _ := os.ReadDir(dirPath)
			if len(l) == 0 {
				os.RemoveAll(dirPath)
			}
		}
	}
}
