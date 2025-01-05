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
	VMR_VERSIONS_ENV    = "VMR_VERSIONS"
	VMR_VERSIONS_PREFIX = `%VMR_VERSIONS%`
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

func GetMingwBashProfilePath() (path string) {
	homeDir, _ := os.UserHomeDir()
	if utils.IsMingWBash() {
		homeDir = utils.ConvertWindowsPathToMingwPath(homeDir)
	}
	path = filepath.Join(homeDir, ".bashrc")
	return
}

const (
	MingwBashExportPattern = `export PATH="$PATH:%s"`
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

// Add env to $HOME/.bashrc
func AddPathEnv(pathEnv string, oldContent string) (newContent string) {
	newContent = oldContent
	if strings.HasPrefix(pathEnv, VersionsDir) {
		exportLine := fmt.Sprintf(
			MingwBashExportPattern,
			utils.ConvertWindowsPathToMingwPath(pathEnv),
		)
		newContent = AddExportLine(newContent, exportLine)
	} else if strings.HasPrefix(pathEnv, VMR_VERSIONS_PREFIX) {
		exportLine := fmt.Sprintf(
			MingwBashExportPattern,
			strings.ReplaceAll(pathEnv, VMR_VERSIONS_PREFIX, utils.ConvertWindowsPathToMingwPath(VersionsDir)),
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
