package sh

import (
	"io/fs"
	"os"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
)

const (
	Bash = "bash"
	Zsh  = "zsh"
	Fish = "fish"
)

const (
	ModePerm         fs.FileMode = 0o644
	VMDisableEnvName string      = "VM_DISABLE"
	vmEnvFileName    string      = "vmr"
)

type Sheller interface {
	ConfPath() string
	VMEnvConfPath() string
	WriteVMEnvToShell()
	PackPath(path string) string
	PackEnv(key, value string) string
}

func FormatPathString(p string) (formattedPath string) {
	formattedPath = p
	if runtime.GOOS != gutils.Windows {
		homeDir, _ := os.UserHomeDir()
		if strings.HasPrefix(p, homeDir) {
			formattedPath = strings.ReplaceAll(p, homeDir, "~")
		}
	}
	return
}
