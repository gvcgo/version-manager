package sh

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/version-manager/pkgs/conf"
)

type BashShell struct{}

func NewBashShell() *BashShell {
	return &BashShell{}
}

func (b *BashShell) ConfPath() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".bashrc")
}

func (b *BashShell) VMEnvConfPath() string {
	installPath := conf.GetVersionManagerWorkDir()
	return filepath.Join(installPath, fmt.Sprintf("%s.sh", vmEnvFileName))
}

func (b *BashShell) WriteVMEnvToShell() {
	installPath := conf.GetVersionManagerWorkDir()
	vmEnvConfPath := b.VMEnvConfPath()
	envStr := fmt.Sprintf(vmEnvZsh, installPath, installPath)
	_ = os.WriteFile(vmEnvConfPath, []byte(envStr), ModePerm)

	shellConfig := b.ConfPath()
	content, _ := os.ReadFile(shellConfig)
	data := string(content)

	home, _ := os.UserHomeDir()
	vmEnvConfPath = strings.ReplaceAll(vmEnvConfPath, home, "~")
	sourceStr := fmt.Sprintf(shellContent, VMDisableEnvName, vmEnvConfPath)
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

func (b *BashShell) PackPath(path string) string {
	return fmt.Sprintf("export PATH=%s:PATH", path)
}

func (b *BashShell) PackEnv(key, value string) string {
	if value == "" {
		return fmt.Sprintf("export %s=", key)
	}
	return fmt.Sprintf("export %s=%s", key, value)
}
