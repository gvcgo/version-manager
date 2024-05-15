package sh

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/version-manager/pkgs/conf"
)

const vmEnvZsh = `# cd hook start
if [ -z $(alias|grep cdhook) ]; then
	cdhook() {
		if [ -d "$1" ];then
			cd "$1"
			vmr use -E
		fi
	}
	alias cd='cdhook'
fi
# cd hook end

export PATH=%s:%s/bin:$PATH
`
const shellContent = `# vmr envs
[[ ! -f %s ]] || source %s
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
	installPath := conf.GetVersionManagerWorkDir()
	return filepath.Join(installPath, fmt.Sprintf("%s.sh", vmEnvFileName))
}

func (z *ZshShell) WriteVMEnvToShell() {
	installPath := conf.GetVersionManagerWorkDir()
	vmEnvConfPath := z.VMEnvConfPath()
	envStr := fmt.Sprintf(vmEnvZsh, installPath, installPath)
	_ = os.WriteFile(vmEnvConfPath, []byte(envStr), 0o644)

	shellConfig := z.ConfPath()
	content, _ := os.ReadFile(shellConfig)
	data := string(content)

	home, _ := os.UserHomeDir()
	vmEnvConfPath = strings.ReplaceAll(vmEnvConfPath, home, "~")
	sourceStr := fmt.Sprintf(shellContent, vmEnvConfPath, vmEnvConfPath)
	if strings.Contains(data, sourceStr) {
		return
	}

	if data == "" {
		data = sourceStr
	} else {
		data = data + "\n" + sourceStr
	}
	_ = os.WriteFile(shellConfig, []byte(data), 0o644)
}

func (z *ZshShell) PackPath(path string) string {
	return fmt.Sprintf("export PATH=%s:PATH", path)
}

func (z *ZshShell) PackEnv(key, value string) string {
	if value == "" {
		return fmt.Sprintf("export %s=", key)
	}
	return fmt.Sprintf("export %s=%s", key, value)
}
