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

// Create symbolic links for directories.
func CreateSymLink(oldname, newname string) (err error) {
	if runtime.GOOS != gutils.Windows {
		err = os.Symlink(oldname, newname)
	} else {
		// Windows
		cmds := []string{
			"cmd",
			"/c",
			"mklink",
			"/j",
			newname,
			oldname,
		}
		homeDir, _ := os.UserHomeDir()
		_, err = gutils.ExecuteSysCommand(true, homeDir, cmds...)
	}
	return
}

func IsMingWBash() bool {
	if runtime.GOOS != gutils.Windows {
		return false
	}
	return strings.Contains(os.Getenv("SHELL"), "bash")
}

func ConvertWindowsPathToMingwPath(originalPath string) (newPath string) {
	if originalPath == "" {
		return
	}
	newPath = strings.ReplaceAll(originalPath, `\`, "/")
	newPath = strings.ReplaceAll(newPath, ":", "")
	dList := strings.Split(newPath, "/")
	diskName := strings.ToLower(dList[0])
	dList = append([]string{diskName}, dList[1:]...)
	newPath = "/" + strings.Join(dList, "/")
	return
}
