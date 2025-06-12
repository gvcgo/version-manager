package utils

import "os"

func PathIsExist(path string) bool {
	_, _err := os.Stat(path)
	if _err == nil {
		return true
	}
	return false
}
