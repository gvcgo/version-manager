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
		_, err = gutils.ExecuteSysCommand(true, homeDir, "cmd", "/c", "start", dUrl)
	case gutils.Linux:
		_, err = gutils.ExecuteSysCommand(true, homeDir, "xdg-open", dUrl)
	case gutils.Darwin:
		_, err = gutils.ExecuteSysCommand(true, homeDir, "open", dUrl)
	default:
		err = fmt.Errorf("unsupported platform")
	}
	return err
}
