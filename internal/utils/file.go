package utils

import (
	"io"
	"os"
)

func PathIsExist(path string) bool {
	_, _err := os.Stat(path)
	if _err == nil {
		return true
	}
	return false
}

func Closeq(v any) {
	if c, ok := v.(io.Closer); ok {
		silently(c.Close())
	}
}

func silently(_ ...any) {}
