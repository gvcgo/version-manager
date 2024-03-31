package terminal

import (
	"fmt"
	"os"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

/*
Prepares envs for terminal.
*/
func AddEnv(key, value string) {
	if strings.ToLower(key) == "path" {
		AddToPath(value)
	} else {
		os.Setenv(key, value)
	}
}

func AddToPath(value string) {
	pathStr := os.Getenv("PATH")
	if runtime.GOOS == gutils.Windows {
		pathStr = fmt.Sprintf("%s;%s", value, pathStr)
	} else {
		pathStr = fmt.Sprintf("%s:%s", value, pathStr)
	}
	os.Setenv("PATH", pathStr)
}
