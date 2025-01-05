package completions

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/shell"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
Setup auto-completions for powershell, bash, zsh, fish.
*/
const PowershellScript = `# VMR Completions
Import-Module -Name "%s"
# VMR Completions`
const OtherShellScript = `# VMR Completions
. %s
# VMR Completions`

func GetBinaryPath() string {
	p, _ := os.Executable()
	if utils.IsMingWBash() {
		p = utils.ConvertWindowsPathToMingwPath(p)
	}
	return p
}

func getCompletionScriptContent() string {
	binPath := GetBinaryPath()
	shellName := "powershell"
	if runtime.GOOS != gutils.Windows {
		shellName = gutils.GetShell()
	}
	if utils.IsMingWBash() {
		shellName = "bash"
	}
	homeDir, _ := os.UserHomeDir()
	content := ""
	if b, err := gutils.ExecuteSysCommand(true, homeDir, binPath, "completion", shellName); err == nil {
		content = b.String()
	}
	return content
}

func writeCompletionScript() (sPath string) {
	content := getCompletionScriptContent()
	sPath = filepath.Join(cnf.GetVMRWorkDir(), "vmr_completions.ps1")
	if runtime.GOOS != gutils.Windows || utils.IsMingWBash() {
		sPath = filepath.Join(cnf.GetVMRWorkDir(), "vmr_completions.sh")
	}
	if content == "" {
		return
	}
	if err := os.WriteFile(sPath, []byte(content), os.ModePerm); err != nil {
		return
	}
	if utils.IsMingWBash() {
		sPath = utils.ConvertWindowsPathToMingwPath(sPath)
	}
	return sPath
}

func AddCompletionScriptToShellProfile() {
	sheller := shell.NewShell()
	shellProfilePath := sheller.ConfPath()
	if shellProfilePath == "" {
		return
	}
	if utils.IsMingWBash() {
		homeDir, _ := os.UserHomeDir()
		shellProfilePath = filepath.Join(homeDir, ".bashrc")
	}

	scriptPath := writeCompletionScript()
	if scriptPath == "" {
		return
	}

	// including mingw bash shell.
	shellScript := OtherShellScript
	if runtime.GOOS == gutils.Windows && !utils.IsMingWBash() {
		// powershell.
		shellScript = PowershellScript
	}
	completionScrit := fmt.Sprintf(shellScript, scriptPath)
	oldProfileContent, err := os.ReadFile(shellProfilePath)
	if err != nil || strings.Contains(string(oldProfileContent), completionScrit) {
		return
	}
	newProfileContent := string(oldProfileContent) + "\n" + completionScrit

	// write new profile content.
	os.WriteFile(shellProfilePath, []byte(newProfileContent), os.ModePerm)
}
