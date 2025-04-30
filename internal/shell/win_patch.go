//go:build windows

package shell

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	VMR_VERSIONS_ENV    = "_VMR_VERSIONS"
	VMR_VERSIONS_PREFIX = `%_VMR_VERSIONS%`
)

var VersionsDir = cnf.GetVersionsDir()

func TidyWindowsPathEnv(pathStr string) (newPath string) {
	if os.Getenv(VMR_VERSIONS_ENV) != VersionsDir {
		shell := NewShell()
		shell.SetEnv(VMR_VERSIONS_ENV, VersionsDir)
	}

	// Handle Path envs for Mingw bash shell.
	HandleVmrRelatedPathEnvsAlreadyExist()
	AddOnePathEnv(pathStr)
	// Patch cd hook for Mingw bash shell.
	PatchVmrCdHookForMingwBash()

	newPath = pathStr
	if strings.Contains(pathStr, VersionsDir) {
		newPath = strings.ReplaceAll(pathStr, VersionsDir, VMR_VERSIONS_PREFIX)
	}
	return
}

/*
Handle Path envs for Mingw bash shell.
*/

func GetVmrRelatedPathEnvsAlreadyExist() (result []string) {
	pathStr := os.Getenv("PATH")
	result = strings.Split(pathStr, ";")
	return
}

/*
Mingw Bash Shell profile path.
*/
func GetMingwBashProfilePath() (path string) {
	homeDir, _ := os.UserHomeDir()
	path = filepath.Join(homeDir, ".bashrc")
	return
}

const (
	MingwBashExportPattern = `export PATH="${PATH}:%s"`
)

func HandleVmrRelatedPathEnvsAlreadyExist() {
	mingwBashProfilePath := GetMingwBashProfilePath()
	content, _ := os.ReadFile(mingwBashProfilePath)
	contentStr := string(content)

	for _, pathEnv := range GetVmrRelatedPathEnvsAlreadyExist() {
		contentStr = AddPathEnv(pathEnv, contentStr)
	}
	os.WriteFile(mingwBashProfilePath, []byte(contentStr), os.ModePerm)
}

func AddOnePathEnv(pathEnv string) {
	mingwBashProfilePath := GetMingwBashProfilePath()
	content, _ := os.ReadFile(mingwBashProfilePath)
	contentStr := string(content)
	contentStr = AddPathEnv(pathEnv, contentStr)
	os.WriteFile(mingwBashProfilePath, []byte(contentStr), os.ModePerm)
}

// Add env to $HOME/.bashrc content.
func AddPathEnv(pathEnv string, oldContent string) (newContent string) {
	newContent = oldContent
	if strings.HasPrefix(pathEnv, VersionsDir) {
		exportLine := fmt.Sprintf(
			MingwBashExportPattern,
			utils.ConvertWindowsPathToMingwPath(pathEnv),
		)
		newContent = AddExportLine(newContent, exportLine)
	} else if strings.HasPrefix(pathEnv, VMR_VERSIONS_PREFIX) {
		p := strings.ReplaceAll(pathEnv, VMR_VERSIONS_PREFIX, VersionsDir)
		exportLine := fmt.Sprintf(
			MingwBashExportPattern,
			utils.ConvertWindowsPathToMingwPath(p),
		)
		newContent = AddExportLine(newContent, exportLine)
	} else if strings.HasPrefix(pathEnv, utils.ConvertWindowsPathToMingwPath(VersionsDir)) {
		exportLine := fmt.Sprintf(
			MingwBashExportPattern,
			pathEnv,
		)
		newContent = AddExportLine(newContent, exportLine)
	}
	return
}

func AddExportLine(oldContent string, exportLine string) (newContent string) {
	if strings.Contains(oldContent, exportLine) {
		newContent = oldContent
	} else if len(oldContent) == 0 {
		newContent = exportLine
	} else {
		newContent = oldContent + "\n" + exportLine
	}
	return
}

/*
cd hook for Mingw bash shell.
*/
const VmrMingwBashCdHook = `# cd hook start
export PATH="%s:${PATH}"

if ! alias | grep -q cdhook; then
	cdhook() {
		if [ $# -eq 0 ]; then
			cd || true
		else
			cd "$@" && vmr use -E
		fi
	}
	alias cd='cdhook'
fi

if [ -z "${VMR_CD_INIT:-}" ]; then
        VMR_CD_INIT="vmr_cd_init"
        cd "$(pwd)" || true
fi
# cd hook end`

// add cd hook to $HOME/.bashrc content.
func PatchVmrCdHookForMingwBash() {
	mingwBashProfilePath := GetMingwBashProfilePath()
	content, _ := os.ReadFile(mingwBashProfilePath)
	contentStr := string(content)

	vmrInstallDir := utils.ConvertWindowsPathToMingwPath(cnf.GetVMRWorkDir())

	cdHook := fmt.Sprintf(VmrMingwBashCdHook, vmrInstallDir)
	if !strings.Contains(contentStr, cdHook) {
		if contentStr == "" {
			contentStr = cdHook
		} else {
			contentStr = contentStr + "\n" + cdHook
		}
	}

	os.WriteFile(mingwBashProfilePath, []byte(contentStr), os.ModePerm)
}
