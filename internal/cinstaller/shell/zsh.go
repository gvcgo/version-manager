package shell

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/version-manager/internal/cnf"
)

type Zsh struct{}

func NewZsh() *Zsh {
	return &Zsh{}
}

func (z *Zsh) ConfPath() string {
	homeDir, _ := os.UserHomeDir()
	return filepath.Join(homeDir, ".zshrc")
}

func (z *Zsh) VMREnvConfPath() string {
	workDir := cnf.GetVMRWorkDir()
	return filepath.Join(
		workDir,
		fmt.Sprintf("%s.sh", VMREnvFileName),
	)
}

func (z *Zsh) Init() {
	// TODO: add cd hook, etc.
}

func (z *Zsh) PackPath(pathS string) string {
	return fmt.Sprintf(`export PATH=%s:"${PATH}"`, pathS)
}

func (z *Zsh) PackEnv(key, value string) string {
	if value == "" {
		return fmt.Sprintf("export %s=", key)
	}
	return fmt.Sprintf("export %s=%s", key, value)
}
