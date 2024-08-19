package sh

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/version-manager/internal/cnf"
)

/*
This is a hook for command "cd".

When you use "cd" command, cdhook will be executed.
In cdhook, it will cd to the target directory and then try to execute "vmr use -E".
The command "vmr use -E" will automatically find the .vmr.lock file, and add corresponding versions of an SDK to the envs.
*/
const vmEnvZsh = `# cd hook start
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

/*
internal/terminal/terminal.go line:90

$VM_DISABLE is an env for the Session Mode of vmr.
It will stop the Shell from loading the envs for SDKs repeatedly.
*/
const shellContent = `# vm_envs start
if [ -z "$%s" ]; then
    . %s
fi
# vm_envs end
`

type ZshShell struct{}

func NewZshShell() *ZshShell {
	return &ZshShell{}
}

func (z *ZshShell) ConfPath() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".zshrc")
}

func (z *ZshShell) VMEnvConfPath() string {
	installPath := cnf.GetVMRWorkDir()
	return filepath.Join(installPath, fmt.Sprintf("%s.sh", VMEnvFileName))
}

func (z *ZshShell) WriteVMEnvToShell() {
	installPath := cnf.GetVMRWorkDir()
	vmEnvConfPath := z.VMEnvConfPath()

	// content, _ := os.ReadFile(vmEnvConfPath)
	// oldEnvStr := strings.TrimSpace(string(content))
	envStr := fmt.Sprintf(vmEnvZsh, FormatPathString(installPath))
	vmrEnvPath := fmt.Sprintf(`export PATH=%s:"${PATH}"`, FormatPathString(installPath))
	UpdateVMRShellFile(vmEnvConfPath, vmrEnvPath, envStr)
	// if !strings.Contains(oldEnvStr, envStr) {
	// 	if oldEnvStr != "" {
	// 		envStr = envStr + "\n" + oldEnvStr
	// 	}
	// 	_ = os.WriteFile(vmEnvConfPath, []byte(envStr), ModePerm)
	// }
	shellConfig := z.ConfPath()
	content, _ := os.ReadFile(shellConfig)
	data := string(content)

	sourceStr := fmt.Sprintf(shellContent, VMDisableEnvName, FormatPathString(vmEnvConfPath))
	if strings.Contains(data, strings.TrimSpace(sourceStr)) {
		return
	}

	if data == "" {
		data = sourceStr
	} else {
		data = data + "\n" + sourceStr
	}
	_ = os.WriteFile(shellConfig, []byte(data), ModePerm)
}

func (z *ZshShell) PackPath(path string) string {
	return fmt.Sprintf(`export PATH=%s:"${PATH}"`, path)
}

func (z *ZshShell) PackEnv(key, value string) string {
	if value == "" {
		return fmt.Sprintf("export %s=", key)
	}
	return fmt.Sprintf("export %s=%s", key, value)
}
