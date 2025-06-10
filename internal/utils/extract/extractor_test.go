package extract

import (
	"fmt"
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
		return
	}
	etr := New(path, dir)
	etr.SetCompressedSingleExe() // Decompress executeable files only
	err := etr.Unarchive()
	if err != nil {
		fmt.Println(err)
	}
}
