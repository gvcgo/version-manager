package post

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
)

func init() {
	RegisterPostInstallHandler(ClojureSDKName, PostInstallForClojure)
}

const (
	ClojureSDKName string = "clojure"
)

func HandleClojureOnWindows(installDir string) {
	// TODO:
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

func PostInstallForClojure(versionName string, version download.Item) {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf("%s_versions", ClojureSDKName))
	clojureInstallDir := filepath.Join(d, fmt.Sprintf("%s-%s", ClojureSDKName, versionName))
	if runtime.GOOS != gutils.Windows {
		HandleClojureOnUnix(clojureInstallDir)
	} else {
		HandleClojureOnWindows(clojureInstallDir)
	}
}
