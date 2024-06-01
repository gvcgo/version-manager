//go:build windows

package utils

import (
	"os"
	"syscall"
	"time"
)

func GetFileLastModifiedTime(fPath string) int64 {
	fInfo, _ := os.Stat(fPath)
	if fInfo == nil {
		return 0
	}
	info := fInfo.Sys().(*syscall.Win32FileAttributeData)
	if info == nil {
		return 0
	}
	return info.LastAccessTime.Nanoseconds() / int64(time.Second)
}
