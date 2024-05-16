package sh

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/version-manager/pkgs/conf"
)

/*
https://fishshell.com/docs/current/cmds/function.html

-v VARIABLE_NAME or --on-variable VARIABLE_NAME
Run this function when the variable VARIABLE_NAME changes value. Note that fish makes no guarantees on any particular timing or even that the function will be run for every single set. Rather it will be run when the variable has been set at least once, possibly skipping some values or being run when the variable has been set to the same value (except for universal variables set in other shells - only changes in the value will be picked up for those).
*/
const vmEnvFish = `# cd hook start
function _vmr_cdhook --on-variable="PWD" --description "version manager cd hook"
	if type -q vmr
        vmr use -E
	end
end
# cd hook end

fish_add_path --global %s %s/bin
`

/*
internal/terminal/terminal.go line:90

$VM_DISABLE is an env for the Session Mode of vmr.
It will stop the Shell from loading the envs for SDKs repeatedly.
*/
const fishShellContent = `# vm_envs start
if not test $%s 
    . %s
end
# vm_envs end`

type FishShell struct{}

func NewFishShell() *FishShell {
	return &FishShell{}
}

func (f *FishShell) ConfPath() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".config/fish/config.fish")
}

func (f *FishShell) VMEnvConfPath() string {
	installPath := conf.GetVersionManagerWorkDir()
	return filepath.Join(installPath, fmt.Sprintf("%s.fish", vmEnvFileName))
}

func (f *FishShell) WriteVMEnvToShell() {
	installPath := conf.GetVersionManagerWorkDir()
	vmEnvConfPath := f.VMEnvConfPath()
	envStr := fmt.Sprintf(vmEnvFish, installPath, installPath)
	_ = os.WriteFile(vmEnvConfPath, []byte(envStr), ModePerm)

	shellConfig := f.ConfPath()
	content, _ := os.ReadFile(shellConfig)
	data := string(content)

	home, _ := os.UserHomeDir()
	vmEnvConfPath = strings.ReplaceAll(vmEnvConfPath, home, "~")
	sourceStr := fmt.Sprintf(fishShellContent, VMDisableEnvName, vmEnvConfPath)
	if strings.Contains(data, sourceStr) {
		return
	}

	if data == "" {
		data = sourceStr
	} else {
		data = data + "\n" + sourceStr
	}
	_ = os.WriteFile(shellConfig, []byte(data), ModePerm)
}

func (f *FishShell) PackPath(path string) string {
	return fmt.Sprintf("fish_add_path --global %s", path)
}

func (f *FishShell) PackEnv(key, value string) string {
	if value == "" {
		return fmt.Sprintf("set --global %s ", key)
	}
	return fmt.Sprintf("set --global %s %s", key, value)
}
