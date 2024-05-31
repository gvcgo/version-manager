package self

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/erikgeiser/promptkit/confirmation"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/shell"
)

type OldConfig struct {
	ProxyURI           string `json:"proxy_uri"`
	ReverseProxy       string `json:"reverse_proxy"`
	AppInstallationDir string `json:"app_installation_dir"`
}

var OldShellRC string = `# vm_envs start
if [ -z $VM_DISABLE ]; then
    . ~/.vm/vmr.sh
fi
# vm_envs end`

/*
Handle old versions.
*/
func DetectAndRemoveOldVersions() {
	homeDir, _ := os.UserHomeDir()
	oldWorkDir := filepath.Join(homeDir, ".vm")
	binName := "vmr"
	if runtime.GOOS == gutils.Windows {
		binName = "vmr.exe"
	}
	oldBinary := filepath.Join(oldWorkDir, binName)
	oldConfPath := filepath.Join(oldWorkDir, "config.json")
	if ok, _ := gutils.PathIsExist(oldBinary); !ok {
		return
	}

	// confirm to remove the old versions.
	fmt.Println(gprint.RedStr("An old version of VMR is detected!"))
	promptStr := gprint.YellowStr("Do you wanna remove the old VMR and all the SDKs installed by it?")
	input := confirmation.New(promptStr, confirmation.NewValue(false))
	input.Template = confirmation.TemplateYN
	input.ResultTemplate = confirmation.ResultTemplateYN
	input.KeyMap.SelectYes = append(input.KeyMap.SelectYes, "+")
	input.KeyMap.SelectNo = append(input.KeyMap.SelectNo, "-")
	ok, _ := input.RunPrompt()
	if !ok {
		fmt.Println(gprint.CyanStr("Installation of new version for VMR is aborted."))
		os.Exit(0)
	}

	// remove the old VMR and all the SDKs installed by it.
	oldConfig := &OldConfig{}
	oldContent, _ := os.ReadFile(oldConfPath)
	json.Unmarshal(oldContent, oldConfig)

	oldSDKInstallationPath := oldWorkDir
	if oldConfig.AppInstallationDir != "" {
		oldSDKInstallationPath = oldConfig.AppInstallationDir
	}

	if oldSDKInstallationPath != "" {
		os.RemoveAll(oldSDKInstallationPath)
	}

	if runtime.GOOS == gutils.Windows {
		sh := shell.NewShell()
		pathEnv := os.Getenv("PATH")
		for _, p := range strings.Split(pathEnv, ";") {
			if strings.HasPrefix(p, oldSDKInstallationPath) || strings.HasPrefix(p, oldWorkDir) {
				sh.UnsetPath(p)
			}
		}
	} else {
		sh := shell.NewShell()
		shellConf := sh.ConfPath()
		content, _ := os.ReadFile(shellConf)
		if len(content) > 0 {
			s := strings.ReplaceAll(string(content), OldShellRC, "")
			os.WriteFile(shellConf, []byte(s), os.ModePerm)
		}
	}

	os.RemoveAll(oldWorkDir)
}

/*
TODO:
Preparation for removing the current version of VMR.
*/
