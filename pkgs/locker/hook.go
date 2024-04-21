package locker

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/envs"
	"github.com/gvcgo/version-manager/pkgs/conf"
)

/*
Hook cd command for shells.
*/

// for Bash/Zsh
const ShellHook = `cdhook() {
    if [ -d "$1" ];then
        cd "$1"
        vmr use -E
    fi
}

alias cd='cdhook'`

// for Powershell
const PowershellHook = `function cdhook {
    $TRUE_FALSE=(Test-Path $args[0])
    if ( $TRUE_FALSE -eq "True" )
    {
        cd $args[0]
        vmr use -E
    }
}

Set-Alias cd cdhook`

func CdHookForUnix() {
	envFilePath := filepath.Join(conf.GetVersionManagerWorkDir(), envs.ShellFileName)
	data, _ := os.ReadFile(envFilePath)
	content := strings.TrimSpace(string(data))
	if !strings.Contains(content, ShellHook) {
		content = fmt.Sprintf("%s\n%s", ShellHook, content)
	}
	os.WriteFile(envFilePath, []byte(content), os.ModePerm)
}

func CdHookForWindows() {
	bf, _ := gutils.ExecuteSysCommand(true, "", "powershell", "echo", "$profile")
	if bf != nil {
		psConfPath := bf.String()
		if psConfPath == "" {
			return
		}
		if ok, _ := gutils.PathIsExist(psConfPath); !ok {
			gutils.ExecuteSysCommand(false, "", "New-Item", "-Type", "file", "-Force", "$profile")
		}
		data, _ := os.ReadFile(psConfPath)
		content := strings.TrimSpace(string(data))
		if !strings.Contains(content, PowershellHook) {
			content = fmt.Sprintf("%s\n%s", PowershellHook, content)
		}
		os.WriteFile(psConfPath, []byte(content), os.ModePerm)
	}
}

func AddCdHook() {
	if runtime.GOOS == gutils.Windows {
		CdHookForWindows()
		return
	}
	CdHookForUnix()
}
