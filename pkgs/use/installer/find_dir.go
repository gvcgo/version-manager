package installer

import (
	"os"
	"path/filepath"
	"strings"
)

/*
Find home dir of unzipped directories.
*/
type HomeDirFinder struct {
	Home      string
	FlagFiles []string // unique file names that only exists in Home Dir.
}

func NewFinder(flagFiles ...string) (h *HomeDirFinder) {
	h = &HomeDirFinder{
		FlagFiles: flagFiles,
	}
	return
}

func (h *HomeDirFinder) Find(startDir string) {
	if h.Home != "" {
		return
	}
	if dList, err := os.ReadDir(startDir); err == nil {

		// Get all filenames in current dir.
		fileNames := ""
		for _, d := range dList {
			// if !d.IsDir() {
			// 	fileNames += d.Name()
			// }
			// including dir names.
			fileNames += d.Name()
		}

		// Test current dir.
		ok := true
		for _, ff := range h.FlagFiles {
			if !strings.Contains(fileNames, ff) {
				ok = false
			}
		}

		if ok {
			h.Home = startDir
		} else {
			// If test failed, continue to test subdirs.
			for _, d := range dList {
				if d.IsDir() {
					h.Find(filepath.Join(startDir, d.Name()))
				}
			}
		}
	}
}
