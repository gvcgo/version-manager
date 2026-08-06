package sh

import (
	"io/fs"
	"os"
	"regexp"
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
	VMEnvFileName    string      = "vmr"
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

/*
Update vmr.sh or vmr.fish
*/
var ShellRegExp = regexp.MustCompile(`# cd hook start[\w\W]+# cd hook end`)

func UpdateVMRShellFile(fPath, vmrPathEnv, newHookContent string) {
	oldData, _ := os.ReadFile(fPath)
	oldContent := string(oldData)
	if oldContent == "" {
		os.WriteFile(fPath, []byte(newHookContent), ModePerm)
		return
	}
	oldHookContent := ShellRegExp.FindString(oldContent)

	if !strings.Contains(oldHookContent, vmrPathEnv) {
		oldContent = strings.ReplaceAll(oldContent, vmrPathEnv, "")
	}

	if oldHookContent != "" {
		oldContent = strings.ReplaceAll(oldContent, oldHookContent, newHookContent)
	} else {
		oldContent = newHookContent + "\n" + oldContent
	}
	_ = os.WriteFile(fPath, []byte(strings.TrimSpace(oldContent)), ModePerm)
}
