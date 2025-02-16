package post

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
)

func init() {
	RegisterPostInstallHandler(ClojureSDKName, PostInstallForClojure)
}

const (
	ClojureSDKName string = "clojure"
)

var clojureEnvForWindows string = `
$CLJ_CONFIG="%s"
Import-Module %s
Invoke-Clojure $args
`

func HandleClojureOnWindows(installDir string) {
	/*
		$CLJ_CONFIG="config path"
		Import-Module $INSTALL_PATH\ClojureTools.psm1
		Invoke-Clojure $args
	*/
	binDir := filepath.Join(installDir, "bin")
	os.MkdirAll(binDir, os.ModePerm)
	powershellScriptModule := filepath.Join(installDir, "ClojureTools.psm1")
	if ok, _ := gutils.PathIsExist(powershellScriptModule); ok {
		confPath := filepath.Join(installDir, "config")
		os.MkdirAll(confPath, os.ModePerm)
		data := fmt.Sprintf(clojureEnvForWindows, confPath, powershellScriptModule)
		newScriptPath := filepath.Join(binDir, "clojure.ps1")
		os.WriteFile(newScriptPath, []byte(data), os.ModePerm)
		newScriptPath = filepath.Join(binDir, "clj.ps1")
		os.WriteFile(newScriptPath, []byte(data), os.ModePerm)
	}
}

func addXforMod(srcPath string) {
	gutils.ExecuteSysCommand(
		true,
		"",
		"chmod",
		"+x",
		srcPath,
	)
}

var clojureScriptFlagForUnix string = "install_dir=PREFIX"
var clojureEnvForUnix string = `install_dir=%s
CLJ_CONFIG=%s`

func HandleClojureOnUnix(installDir string) {
	libexecDir := filepath.Join(installDir, "libexec")
	os.MkdirAll(libexecDir, os.ModePerm)
	dList, _ := os.ReadDir(installDir)
	for _, dd := range dList {
		if !dd.IsDir() && strings.HasSuffix(dd.Name(), ".jar") {
			jarPath := filepath.Join(installDir, dd.Name())
			gutils.CopyAFile(jarPath, libexecDir)
		}
	}

	binDir := filepath.Join(installDir, "bin")
	os.MkdirAll(binDir, os.ModePerm)
	content, _ := os.ReadFile(filepath.Join(installDir, "clojure"))
	data := string(content)
	if strings.Contains(data, clojureScriptFlagForUnix) {
		confPath := filepath.Join(installDir, "config")
		os.MkdirAll(confPath, os.ModePerm)
		newEnv := fmt.Sprintf(clojureEnvForUnix, installDir, confPath)
		data := strings.ReplaceAll(data, clojureScriptFlagForUnix, newEnv)

		newScriptPath := filepath.Join(binDir, "clojure")
		os.WriteFile(newScriptPath, []byte(data), os.ModePerm)
		addXforMod(newScriptPath)
	}

	content, _ = os.ReadFile(filepath.Join(installDir, "clj"))
	data = string(content)
	newScriptPath := filepath.Join(binDir, "clj")
	os.WriteFile(newScriptPath, []byte(data), os.ModePerm)
	addXforMod(newScriptPath)
}

func PostInstallForClojure(versionName string, version lua_global.Item) {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf("%s_versions", ClojureSDKName))
	clojureInstallDir := filepath.Join(d, fmt.Sprintf("%s-%s", ClojureSDKName, versionName))
	if runtime.GOOS != gutils.Windows {
		HandleClojureOnUnix(clojureInstallDir)
	} else {
		HandleClojureOnWindows(clojureInstallDir)
	}
}
