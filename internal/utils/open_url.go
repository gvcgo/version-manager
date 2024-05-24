package utils

import (
	"fmt"
	"os"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

func OpenURL(dUrl string) error {
	homeDir, _ := os.UserHomeDir()
	var err error
	switch runtime.GOOS {
	case gutils.Windows:
		_, err = gutils.ExecuteSysCommand(false, homeDir, "cmd", "/c", "start", dUrl)
	case gutils.Linux:
		_, err = gutils.ExecuteSysCommand(false, homeDir, "x-www-browser", dUrl)
	case gutils.Darwin:
		_, err = gutils.ExecuteSysCommand(false, homeDir, "open", dUrl)
	default:
		err = fmt.Errorf("unsupported platform")
	}
	return err
}
