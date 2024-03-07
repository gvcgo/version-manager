package utils

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

func GetShellConfigFilePath() string {
	if runtime.GOOS == gutils.Windows {
		return ""
	}
	shellInfo := os.Getenv("SHELL")
	homeDir, _ := os.UserHomeDir()
	if strings.Contains(shellInfo, "zsh") {
		return filepath.Join(homeDir, ".zshrc")
	} else if strings.Contains(shellInfo, "bash") {
		return filepath.Join(homeDir, ".bashrc")
	} else if strings.Contains(shellInfo, "fish") {
		return filepath.Join(homeDir, ".config/fish/config.fish")
	} else {
		return ""
	}
}
