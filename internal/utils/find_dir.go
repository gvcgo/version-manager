package utils

import (
	"os"
	"path/filepath"
	"strings"
)

/*
Find home dir of unzipped directories.
*/
type HomeDirFinder struct {
	home      string
	flagFiles []string // unique file names that only exists in Home Dir.
	exceptDir bool
}

func NewFinder(flagFiles ...string) (h *HomeDirFinder) {
	h = &HomeDirFinder{
		flagFiles: flagFiles,
	}
	return
}

func (h *HomeDirFinder) Find(startDir string) {
	if h.home != "" {
		return
	}
	if dList, err := os.ReadDir(startDir); err == nil {

		// Get all filenames in current dir.
		fileNames := ""
		for _, d := range dList {
			if h.exceptDir {
				if !d.IsDir() {
					fileNames += d.Name()
				}
			} else {
				// including dir names.
				fileNames += d.Name()
			}
		}

		// Test current dir.
		ok := true
		for _, ff := range h.flagFiles {
			if !strings.Contains(fileNames, ff) {
				ok = false
			}
		}

		if ok {
			h.home = startDir
		} else {
			// If test failed, continue to test subdirs.
			for _, d := range dList {
				if d.IsDir() && d.Name() != "__MACOSX" {
					h.Find(filepath.Join(startDir, d.Name()))
				}
			}
		}
	}
}

func (h *HomeDirFinder) SetFlags(flagFiles ...string) {
	h.flagFiles = flagFiles
}

func (h *HomeDirFinder) SetFlagDirExcepted(ok bool) {
	h.exceptDir = ok
}

func (h *HomeDirFinder) Clear() {
	h.flagFiles = []string{}
	h.home = ""
	h.exceptDir = false
}

func (h *HomeDirFinder) GetDirName() string {
	return h.home
}
