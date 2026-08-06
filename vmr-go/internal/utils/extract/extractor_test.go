package extract

import (
	"path/filepath"
	"runtime"
	"testing"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

func TestExtractor(t *testing.T) {
	_, current, _, _ := runtime.Caller(0)
	dir := filepath.Dir(current)
	path := filepath.Join(dir, "test.gz")
	if ok, _ := gutils.PathIsExist(path); !ok {
		t.Skip("test file not found")
	}
	etr := New(path, dir)
	etr.SetCompressedSingleExe() // Decompress executeable files only
	err := etr.Unarchive()
	if err != nil {
		t.Errorf("Error extracting archive: %v", err)
	}
}
