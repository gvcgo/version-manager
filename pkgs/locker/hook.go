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
const ShellHook string = `# cd hook start
%s
if [ -z "$cdalias" ]; then
	cdhook() {
		if [ -d "$1" ];then
			cd "$1"
			vmr use -E
		fi
	}
	alias cd='cdhook'
fi
# cd hook end`

const ShellHookStr string = "cdalias=`alias|grep cdhook`"

// for Powershell
const PowershellHook string = `function cdhook {
    $TRUE_FALSE=(Test-Path $args[0])
    if ( $TRUE_FALSE -eq "True" )
    {
        chdir $args[0]
        vmr use -E
    }
}

function vmrsource {
	$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

Set-Alias -Name cd -Option AllScope -Value cdhook
Set-Alias -Name source -Value vmrsource`

func CdHookForUnix() {
	envFilePath := filepath.Join(conf.GetVersionManagerWorkDir(), envs.ShellFileName)
	data, _ := os.ReadFile(envFilePath)
	content := strings.TrimSpace(string(data))
	var flag string
	if !strings.Contains(content, "# cd hook end") {
		flag = "alias cd='cdhook'"
	} else {
		flag = "# cd hook end"
	}
	if strings.Contains(content, flag) {
		sList := strings.Split(content, flag)
		if len(sList) > 1 {
			content = sList[len(sList)-1]
		}
	}

	shellHook := fmt.Sprintf(ShellHook, ShellHookStr)
	if !strings.Contains(content, shellHook) {
		content = fmt.Sprintf("%s\n%s", shellHook, strings.TrimSpace(content))
	}
	os.WriteFile(envFilePath, []byte(content), os.ModePerm)
}

func CdHookForWindows() {
	/*
		powershell config file path:
		https://learn.microsoft.com/zh-cn/powershell/module/microsoft.powershell.core/about/about_profiles?view=powershell-7.4
		https://www.jb51.net/article/53412.htm

		set-alias:
		https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/set-alias?view=powershell-7.4
	*/
	homeDir, _ := os.UserHomeDir()

	psConfDir := filepath.Join(homeDir,
		"Documents",
		"WindowsPowerShell",
	)
	psConfName := "profile.ps1"

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
