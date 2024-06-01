//go:build linux

package utils

import (
	"os"
	"syscall"
)

func GetFileLastModifiedTime(fPath string) int64 {
	fInfo, _ := os.Stat(fPath)
	if fInfo == nil {
		return 0
	}
	info := fInfo.Sys().(*syscall.Stat_t)
	if info == nil {
		return 0
	}
	return info.Mtim.Sec
}
