package shell

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/version-manager/internal/cnf"
)

type Bash struct{}

func NewBash() *Bash {
	return &Bash{}
}

func (b *Bash) ConfPath() string {
	homeDir, _ := os.UserHomeDir()
	return filepath.Join(homeDir, ".bashrc")
}

func (b *Bash) VMREnvConfPath() string {
	workDir := cnf.GetVMRWorkDir()
	return filepath.Join(
		workDir,
		fmt.Sprintf("%s.sh", VMREnvFileName),
	)
}

func (b *Bash) Init() {
	// TODO: add cd hook, etc.
}

func (b *Bash) PackPath(pathS string) string {
	return fmt.Sprintf("export PATH=%s:$PATH", pathS)
}

func (b *Bash) PackEnv(key, value string) string {
	if value == "" {
		return fmt.Sprintf("export %s=", key)
	}
	return fmt.Sprintf("export %s=%s", key, value)
}
