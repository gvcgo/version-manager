//go:build darwin || linux

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

/*
Unix/Linux envs manager.
*/
type EnvManager struct{}

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

/*
Add shell file to shell confs.
*/
const shellContent string = `# vm_envs start
if [ -z $%s ]; then
    %s
fi
# vm_envs end
`

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

		if strings.Contains(data, envStr) && !strings.Contains(data, "# vm_envs start") {
			data = strings.TrimSpace(strings.ReplaceAll(data, envStr, ""))
		}

		envStr = fmt.Sprintf(shellContent, conf.VMDiableEnvName, envStr)
		if data == "" {
			data = envStr
		} else if !strings.Contains(data, envStr) {
			data = data + "\n\n" + envStr
		}
		os.WriteFile(shellConfFile, []byte(data), os.ModePerm)
	}
}

func (em *EnvManager) UnsetPath() {
	shellFile := filepath.Join(conf.GetVersionManagerWorkDir(), ShellFileName)
	content, _ := os.ReadFile(shellFile)
	data := string(content)
	envStr := fmt.Sprintf(`export PATH=%s:$PATH`, conf.GetAppBinDir())
	if strings.Contains(data, conf.GetAppBinDir()) {
		data = strings.TrimSpace(strings.ReplaceAll(data, envStr, ""))
		data = strings.ReplaceAll(data, "\n\n", "\n")
	}
	os.WriteFile(shellFile, []byte(data), os.ModePerm)
	em.removeShellFileFromShellConfig()
}

func (em *EnvManager) removeShellFileFromShellConfig() {
	shellConfFile := utils.GetShellConfigFilePath()
	if shellConfFile != "" {
		data := ""
		if ok, _ := gutils.PathIsExist(shellConfFile); ok {
			content, _ := os.ReadFile(shellConfFile)
			data = string(content)
		}
		shellFile := filepath.Join(conf.GetVersionManagerWorkDir(), ShellFileName)

		envStr := fmt.Sprintf(`. %s`, shellFile)
		if strings.Contains(data, envStr) {
			data = strings.TrimSpace(strings.ReplaceAll(data, envStr, ""))
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

func (em *EnvManager) UnSet(key string) {
	shellFile := filepath.Join(conf.GetVersionManagerWorkDir(), ShellFileName)
	content, _ := os.ReadFile(shellFile)
	data := string(content)
	for _, line := range strings.Split(data, "\n") {
		if strings.Contains(line, key) {
			data = strings.TrimSpace(strings.ReplaceAll(data, line, ""))
		}
	}
	os.WriteFile(shellFile, []byte(data), os.ModePerm)
}

// Adds new item to $PATH env.
func (em *EnvManager) AddToPath(value string) {
	shellFile := filepath.Join(conf.GetVersionManagerWorkDir(), ShellFileName)
	content, _ := os.ReadFile(shellFile)
	data := string(content)

	envStr := fmt.Sprintf(`export PATH=%s:$PATH`, value)
	if data == "" {
		data = envStr
	} else if !strings.Contains(data, envStr) {
		data = data + "\n" + envStr
	}
	os.WriteFile(shellFile, []byte(data), os.ModePerm)
	em.addShellFileToShellConfig()
}

// Deletes an item from $PATH env.
func (em *EnvManager) DeleteFromPath(value string) {
	shellFile := filepath.Join(conf.GetVersionManagerWorkDir(), ShellFileName)
	content, _ := os.ReadFile(shellFile)
	data := string(content)

	envStr := fmt.Sprintf(`export PATH=%s:$PATH`, value)
	if strings.Contains(data, envStr) {
		data = strings.TrimSpace(strings.ReplaceAll(data, envStr, ""))
		data = strings.ReplaceAll(data, "\n\n", "\n")
	}
	os.WriteFile(shellFile, []byte(data), os.ModePerm)
	em.addShellFileToShellConfig()
}

func (em *EnvManager) CloseKey() {}
