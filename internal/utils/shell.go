package utils

import (
	"os"
	"os/exec"
	"runtime"
	"strings"

	"github.com/gogf/gf/v2/os/genv"
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

const (
	LinuxInstallerApt string = "apt"
	LinuxInstallerYum string = "yum"
	LinuxInstallerDnf string = "dnf"
)

func DNForAPTonLinux() string {
	_, err := gutils.ExecuteSysCommand(true, "", "apt", "--help")
	if err == nil {
		return LinuxInstallerApt
	}
	_, err = gutils.ExecuteSysCommand(true, "", "dnf", "--help")
	if err == nil {
		return LinuxInstallerDnf
	}
	_, err = gutils.ExecuteSysCommand(true, "", "yum", "--help")
	if err == nil {
		return LinuxInstallerYum
	}
	return ""
}

func MoveFileOnUnixSudo(from, to string) error {
	cmd := exec.Command("sudo", "mv", from, to)
	cmd.Env = genv.All()
	cmd.Stderr = os.Stderr
	cmd.Stdout = os.Stdout
	cmd.Stdin = os.Stdin
	return cmd.Run()
}
