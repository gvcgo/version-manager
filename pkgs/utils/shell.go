package utils

import (
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gogf/gf/v2/os/genv"
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

func CopyFileOnUnixSudo(from, to string) error {
	cmd := exec.Command("sudo", "cp", "-R", from, to)
	cmd.Env = genv.All()
	cmd.Stderr = os.Stderr
	cmd.Stdout = os.Stdout
	cmd.Stdin = os.Stdin
	return cmd.Run()
}

func MoveFileOnUnixSudo(from, to string) error {
	cmd := exec.Command("sudo", "mv", from, to)
	cmd.Env = genv.All()
	cmd.Stderr = os.Stderr
	cmd.Stdout = os.Stdout
	cmd.Stdin = os.Stdin
	return cmd.Run()
}

func DNForAPTonLinux() string {
	_, err := gutils.ExecuteSysCommand(true, "", "apt", "--help")
	if err == nil {
		return "apt"
	}
	_, err = gutils.ExecuteSysCommand(true, "", "dnf", "--help")
	if err == nil {
		return "dnf"
	}
	_, err = gutils.ExecuteSysCommand(true, "", "yum", "--help")
	if err == nil {
		return "yum"
	}
	return ""
}
