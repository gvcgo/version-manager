package shell

import (
	"io/fs"
	"os"
	"regexp"
	"runtime"
	"strings"

	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	VMREnvFileName                 = "vmr"
	VMREnvDisableFlag              = "VMR_DISABLE_SDK_ENVS"
	VMREnvFileModePerm fs.FileMode = 0o644
)

func FormatPath(pathS string) string {
	formattedPath := pathS
	if runtime.GOOS != utils.Windows {
		homeDir, _ := os.UserHomeDir()
		if strings.HasPrefix(pathS, homeDir) {
			formattedPath = strings.ReplaceAll(pathS, homeDir, "~")
		}
	}
	return formattedPath
}

/*
This is a hook for command "cd".

When you use "cd" command, cdhook will be executed.
In cdhook, it will cd to the target directory and then try to execute "vmr use -E".
The command "vmr use -E" will automatically find the .vmr.lock file, and add corresponding versions of an SDK to the envs.
*/
const CdHookForBashAndZsh = `# cd hook start
export PATH=%s:"${PATH}"

if [ -z "$(alias|grep cdhook)" ]; then
	cdhook() {
		if [ $# -eq 0 ]; then
			cd
		else
			cd "$@" && vmr use -E
		fi
	}
	alias cd='cdhook'
fi

if [ -z "${VMR_CD_INIT}" ]; then
        VMR_CD_INIT="vmr_cd_init"
        cd "$(pwd)"
fi
# cd hook end`

const CdHookForFish = `# cd hook start
fish_add_path --global %s

function _vmr_cdhook --on-variable="PWD" --description "version manager cd hook"
	if type -q vmr
        vmr use -E
	end
end

if set -q "$VMR_CD_INIT"
	set VMR_CD_INIT "vmr_cd_init"
    cd "$(pwd)"
end
# cd hook end`

const CdHookForPowershell = `# cd hook start
function cdhook {
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
Set-Alias -Name source -Value vmrsource

if ( "" -eq "$env:VMR_CD_INIT" )
{
    $env:VMR_CD_INIT="vmr_cd_init"
    cd "$(-split $(pwd))"
}
# cd hook end`

const CdHookForCmd = ``

var CdHookRegExp = regexp.MustCompile(`# cd hook start[\w\W]+# cd hook end`)

/*
internal/terminal/terminal.go line:90

$VMR_DISABLE_SDK_ENVS is an env for the Session Mode of vmr.
It will stop the Shell from loading the envs for SDKs repeatedly.
*/
const LoadSDKEnvsForBashAndZsh = `# vm_envs start
if [ -z "$%s" ]; then
    . %s
fi
# vm_envs end
`

const LoadSDKEnvsForFish = `# vm_envs start
if not test $%s
    . %s
end
# vm_envs end`

var LoadSDKEnvsRegExp = regexp.MustCompile(`# vm_envs start[\w\W]+# vm_envs end`)
