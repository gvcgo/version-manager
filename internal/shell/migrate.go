package shell

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/shell/sh"
	"github.com/gvcgo/version-manager/pkgs/conf"
)

/*
Miagrates shell configurations.
*/
type ShellMigrator struct {
	shell sh.Sheller
}

func NewShellMigrator() (s *ShellMigrator) {
	var shell sh.Sheller
	shellPath := os.Getenv("SHELL")
	switch {
	case strings.HasSuffix(shellPath, sh.Bash):
		shell = sh.NewBashShell()
	case strings.HasSuffix(shellPath, sh.Zsh):
		shell = sh.NewZshShell()
	case strings.HasSuffix(shellPath, sh.Fish):
		shell = sh.NewFishShell()
	default:
	}
	return &ShellMigrator{shell: shell}
}

// vmr.sh
func (s *ShellMigrator) getvmrdotsh() string {
	shell := sh.NewZshShell()
	return shell.VMEnvConfPath()
}

// vmr.fish
func (s *ShellMigrator) getvmrdotfish() string {
	shell := sh.NewFishShell()
	return shell.VMEnvConfPath()
}

// Old File: vm_env.sh
func (s *ShellMigrator) getvmenvdotsh() string {
	installPath := conf.GetVersionManagerWorkDir()
	return filepath.Join(installPath, fmt.Sprintf("%s.sh", "vm_env"))
}

func (s *ShellMigrator) fishLineToline(line string) (newLine string) {
	line = strings.TrimSpace(line)
	pathPrefix := "fish_add_path --global "
	envPrefix := "set --global "
	if strings.HasPrefix(line, pathPrefix) {
		p := strings.TrimSpace(strings.ReplaceAll(line, pathPrefix, ""))
		pList := strings.Split(p, " ")
		if len(pList) > 0 {
			p = strings.Join(pList, ":")
			newLine = fmt.Sprintf("export PATH=%s:$PATH", p)
		}
	} else if strings.HasPrefix(line, envPrefix) {
		envStr := strings.TrimSpace(strings.ReplaceAll(line, envPrefix, ""))
		eList := strings.Split(envStr, " ")
		if len(eList) == 2 {
			newLine = fmt.Sprintf("export %s=%s", eList[0], eList[1])
		}
	}
	return
}

func (s *ShellMigrator) lineToFishLine(line string) (newLine string) {
	pathPrefix := "export PATH="
	envPrefix := "export "
	if strings.HasPrefix(line, pathPrefix) {
		line = strings.TrimSpace(strings.ReplaceAll(line, pathPrefix, ""))
		line = strings.TrimSuffix(line, ":$PATH")
		pList := strings.Split(line, ":")
		if len(pList) > 0 {
			newLine = fmt.Sprintf("fish_add_path --global %s", strings.Join(pList, " "))
		}
	} else if strings.HasPrefix(line, envPrefix) && !strings.Contains(line, "$PATH") {
		line = strings.TrimSpace(strings.ReplaceAll(line, envPrefix, ""))
		eList := strings.Split(line, "=")
		if len(eList) == 2 {
			newLine = fmt.Sprintf("set --global %s %s", eList[0], eList[1])
		}
	}
	return
}

func (s *ShellMigrator) filterLine(line string) bool {
	if line == "" {
		return false
	}

	installPath := conf.GetVersionManagerWorkDir()
	installBinPath := filepath.Join(installPath, "bin")

	if line == fmt.Sprintf("fish_add_path --global %s", installPath) {
		return false
	}

	if line == fmt.Sprintf("fish_add_path --global %s", installBinPath) {
		return false
	}

	if line == fmt.Sprintf("fish_add_path --global %s", sh.FormatPathString(installPath)) {
		return false
	}

	if line == fmt.Sprintf("fish_add_path --global %s", sh.FormatPathString(installBinPath)) {
		return false
	}

	if line == fmt.Sprintf("fish_add_path --global %s %s", installPath, installBinPath) {
		return false
	}

	if line == fmt.Sprintf("fish_add_path --global %s %s", sh.FormatPathString(installPath), sh.FormatPathString(installBinPath)) {
		return false
	}

	if !strings.HasPrefix(line, "export") {
		return true
	}

	if line == fmt.Sprintf("export PATH=%s:$PATH", installPath) {
		return false
	}

	if line == fmt.Sprintf("export PATH=%s:$PATH", installBinPath) {
		return false
	}

	if line == fmt.Sprintf("export PATH=%s:$PATH", sh.FormatPathString(installPath)) {
		return false
	}

	if line == fmt.Sprintf("export PATH=%s:$PATH", sh.FormatPathString(installBinPath)) {
		return false
	}

	if line == fmt.Sprintf("export PATH=%s:%s:$PATH", sh.FormatPathString(installPath), sh.FormatPathString(installBinPath)) {
		return false
	}

	if line == fmt.Sprintf("export PATH=%s:%s:$PATH", installPath, installBinPath) {
		return false
	}
	return true
}

func (s *ShellMigrator) handle(oldFile string, lineHandler func(string) string) {
	oldData, _ := os.ReadFile(oldFile)
	oldContent := strings.TrimSpace(string(oldData))

	vmrConfFile := s.shell.VMEnvConfPath()
	newData, _ := os.ReadFile(vmrConfFile)
	newContent := strings.TrimSpace(string(newData))

	lines := strings.Split(oldContent, "\n")
	for _, line := range lines {
		l := lineHandler(line)

		if !s.filterLine(l) {
			continue
		}

		if l != "" && !strings.Contains(newContent, l) {
			newContent = newContent + "\n" + l
		}
	}

	_ = os.WriteFile(vmrConfFile, []byte(newContent), sh.ModePerm)
}

// Migrate: vm_env.sh -> vmr.sh
func (s *ShellMigrator) vmEnvToVMR() {
	vmenvFile := s.getvmenvdotsh()
	if ok, _ := gutils.PathIsExist(vmenvFile); !ok {
		return
	}
	s.handle(vmenvFile, strings.TrimSpace)
}

// Migrate: vmr.fish	-> vmr.sh
func (s *ShellMigrator) vmrFishToVMR() {
	vmrdotfishFile := s.getvmrdotfish()
	if ok, _ := gutils.PathIsExist(vmrdotfishFile); !ok {
		return
	}
	s.handle(vmrdotfishFile, s.fishLineToline)
}

// Migrate: vm_env.sh -> vmr.fish
func (s *ShellMigrator) vmEnvToVMRFish() {
	vmenvFile := s.getvmenvdotsh()
	if ok, _ := gutils.PathIsExist(vmenvFile); !ok {
		return
	}
	s.handle(vmenvFile, s.lineToFishLine)
}

// Migrate: vmr.sh -> vmr.fish
func (s *ShellMigrator) vmrToVMRFish() {
	vmrdotshFile := s.getvmrdotsh()
	if ok, _ := gutils.PathIsExist(vmrdotshFile); !ok {
		return
	}
	s.handle(vmrdotshFile, s.lineToFishLine)
}

func (s *ShellMigrator) Migrate() {
	if s.shell == nil {
		return
	}
	switch s.shell.(type) {
	case *sh.BashShell, *sh.ZshShell:
		s.vmEnvToVMR()
		s.vmrFishToVMR()
	case *sh.FishShell:
		s.vmEnvToVMRFish()
		s.vmrToVMRFish()
	default:
	}
}
