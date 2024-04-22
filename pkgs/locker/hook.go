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

alias cdr='cdhook'`

// for Powershell
const PowershellHook = `function cdhook {
    $TRUE_FALSE=(Test-Path $args[0])
    if ( $TRUE_FALSE -eq "True" )
    {
        cd $args[0]
        vmr use -E
    }
}

Set-Alias cdr cdhook`

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
	homeDir, _ := os.UserHomeDir()

	psConfDir := filepath.Join(homeDir,
		"Documents",
		"WindowsPowerShell",
	)
	psConfName := "Microsoft.PowerShell_profile.ps1"

	if ok, _ := gutils.PathIsExist(psConfDir); !ok {
		os.MkdirAll(psConfDir, os.ModePerm)
	}

	psConfPath := filepath.Join(psConfDir, psConfName)

	var content string
	if ok, _ := gutils.PathIsExist(psConfPath); ok {
		data, _ := os.ReadFile(psConfPath)
		content = strings.TrimSpace(string(data))
	}

	if strings.Contains(content, PowershellHook) {
		return
	}

	if content != "" {
		content = fmt.Sprintf("%s\n%s", PowershellHook, content)
	} else {
		content = PowershellHook
	}

	err := os.WriteFile(psConfPath, []byte(content), os.ModePerm)
	if err != nil {
		os.WriteFile("vmr_error.txt", []byte(err.Error()), os.ModePerm)
	}
}

func AddCdHook() {
	if runtime.GOOS == gutils.Windows {
		CdHookForWindows()
	} else {
		CdHookForUnix()
	}
}
