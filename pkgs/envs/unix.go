package envs

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/utils"
)

const (
	ShellFileName string = "shell.sh"
)

/*
Unix/Linux envs manager.
*/
type EnvManager struct {
}

func NewEnvManager() (em *EnvManager) {
	em = &EnvManager{}
	return
}

func (em *EnvManager) SetPath() {
	shellFile := filepath.Join(conf.GetVersionManagerWorkDir(), ShellFileName)
	content, _ := os.ReadFile(shellFile)
	data := string(content)
	envStr := fmt.Sprintf(`export PATH=%s:$PATH`, conf.GetAppBinDir())
	if data == "" {
		data = envStr
	} else if !strings.Contains(data, conf.GetAppBinDir()) {
		data = data + "\n" + envStr
	}
	os.WriteFile(shellFile, []byte(data), os.ModePerm)
	em.addShellFileToShellConfig()
}

func (em *EnvManager) addShellFileToShellConfig() {
	shellConfFile := utils.GetShellConfigFilePath()
	if shellConfFile != "" {
		data := ""
		if ok, _ := gutils.PathIsExist(shellConfFile); ok {
			content, _ := os.ReadFile(shellConfFile)
			data = string(content)
		}
		shellFile := filepath.Join(conf.GetVersionManagerWorkDir(), ShellFileName)

		envStr := fmt.Sprintf(`. %s`, shellFile)
		if data == "" {
			data = envStr
		} else if !strings.Contains(data, envStr) {
			data = data + "\n" + envStr
		}
		os.WriteFile(shellConfFile, []byte(data), os.ModePerm)
	}
}

func (em *EnvManager) Set(key, value string) {
	shellFile := filepath.Join(conf.GetVersionManagerWorkDir(), ShellFileName)
	content, _ := os.ReadFile(shellFile)
	data := string(content)

	envStr := fmt.Sprintf(`export %s=%s`, key, value)
	if data == "" {
		data = envStr
	} else if !strings.Contains(data, envStr) {
		data = data + "\n" + envStr
	}
	os.WriteFile(shellFile, []byte(data), os.ModePerm)
	em.addShellFileToShellConfig()
}
