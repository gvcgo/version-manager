package shell

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/version-manager/internal/cnf"
)

type Fish struct{}

func NewFish() *Fish {
	return &Fish{}
}

func (f *Fish) ConfPath() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".config/fish/config.fish")
}

func (f *Fish) VMEnvConfPath() string {
	workDir := cnf.GetVMRWorkDir()
	return filepath.Join(workDir, fmt.Sprintf("%s.fish", VMREnvFileName))
}

func (f *Fish) Init() {
	// TODO: add cd hook, etc.
}

func (f *Fish) PackPath(pathS string) string {
	return fmt.Sprintf("fish_add_path --global %s", pathS)
}

func (f *Fish) PackEnv(key, value string) string {
	if value == "" {
		return fmt.Sprintf("set --global %s ", key)
	}
	return fmt.Sprintf("set --global %s %s", key, value)
}
