package utils

import (
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

const (
	WinPathSeperator  string = ";"
	UnixPathSeperator string = ":"
)

func JoinPath(pathStr ...string) (s string) {
	if len(pathStr) == 0 {
		return
	}
	seperator := WinPathSeperator
	if runtime.GOOS != gutils.Windows {
		seperator = UnixPathSeperator
	}
	s = strings.Join(pathStr, seperator)
	return
}
