package extract

import (
	"path/filepath"
	"runtime"
	"testing"
)

func TestExtractor(t *testing.T) {
	_, current, _, _ := runtime.Caller(0)
	dir := filepath.Dir(current)
	path := filepath.Join(dir, "test.gz")
	etr := New(path, dir)
	etr.SetCompressedSingleExe() // Decompress executeable files only
	err := etr.Unarchive()
	if err != nil {
		t.Errorf("Error extracting archive: %v", err)
	}
}
