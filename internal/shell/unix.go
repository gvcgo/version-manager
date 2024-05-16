//go:build darwin || linux

package shell

import (
	"os"
	"strings"

	"github.com/gvcgo/version-manager/internal/shell/sh"
)

var _ Sheller = (*Shell)(nil)

type Shell struct {
	sh.Sheller
}

func NewShell() *Shell {
	var shell sh.Sheller
	shellPath := os.Getenv("SHELL")
	switch {
	case strings.HasSuffix(shellPath, sh.Bash):
		shell = sh.NewBashShell()
	case strings.HasSuffix(shellPath, sh.Zsh):
		shell = sh.NewZshShell()
	case strings.HasSuffix(shellPath, sh.Fish):
		shell = sh.NewFishShell()
	}
	return &Shell{shell}
}

func (s *Shell) SetPath(path string) {
	content, _ := os.ReadFile(s.VMEnvConfPath())
	data := string(content)

	path = s.PackPath(path)
	data = strings.ReplaceAll(data, path+"\n", "")
	data = data + "\n" + path
	_ = os.WriteFile(s.VMEnvConfPath(), []byte(data), sh.ModePerm)
}

func (s *Shell) UnsetPath(path string) {
	content, _ := os.ReadFile(s.VMEnvConfPath())
	data := string(content)
	data = strings.ReplaceAll(data, "\n"+s.PackPath(path)+"\n", "")
	_ = os.WriteFile(s.VMEnvConfPath(), []byte(data), sh.ModePerm)
}

func (s *Shell) SetEnv(key, value string) {
	content, _ := os.ReadFile(s.VMEnvConfPath())
	data := string(content)

	env := s.PackEnv(key, value)
	data = strings.ReplaceAll(data, env+"\n", "")
	data = data + "\n" + env
	_ = os.WriteFile(s.VMEnvConfPath(), []byte(data), sh.ModePerm)
}

func (s *Shell) UnsetEnv(key string) {
	content, _ := os.ReadFile(s.VMEnvConfPath())
	data := string(content)
	env := s.PackEnv(key, "")
	for _, line := range strings.Split(env, "\n") {
		if strings.HasPrefix(line, env) {
			data = strings.ReplaceAll(data, line+"\n", "")
		}
	}
	_ = os.WriteFile(s.VMEnvConfPath(), []byte(data), sh.ModePerm)
}

func (s *Shell) Close() {}
