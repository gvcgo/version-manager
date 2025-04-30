package post

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/utils"
)

func init() {
	RegisterPostInstallHandler(MoonbitSdkName, PostInstallForMoonbit)
}

const (
	MoonbitSdkName string = "moonbit"
)

/*
https://cli.moonbitlang.com/cores/core-latest.tar.gz
*/
var MoonbitCoreUrl string = "https://cli.moonbitlang.com/cores/core-latest.tar.gz"

func MoonbitChmod(moonbitInstallDir string) {
	if runtime.GOOS == gutils.Windows {
		return
	}
	dItems, _ := os.ReadDir(moonbitInstallDir)
	for _, d := range dItems {
		if d.IsDir() {
			continue
		}
		if strings.HasPrefix(d.Name(), "lib") {
			continue
		}
		fPath := filepath.Join(moonbitInstallDir, d.Name())
		os.Chmod(fPath, 0755)
	}
}

func PostInstallForMoonbit(versionName string, version lua_global.Item) {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf("%s_versions", MoonbitSdkName))
	moonbitInstallDir := filepath.Join(d, fmt.Sprintf("%s-%s", MoonbitSdkName, versionName))
	if ok, _ := gutils.PathIsExist(moonbitInstallDir); !ok {
		return
	}

	MoonbitChmod(moonbitInstallDir)

	// download core lib.
	coreTarFilePath := filepath.Join(moonbitInstallDir, "core-latest.tar.gz")
	fetcher := cnf.GetFetcher(MoonbitCoreUrl)
	fetcher.Timeout = 30 * time.Minute
	if size := fetcher.GetAndSaveFile(coreTarFilePath, true); size <= 100 {
		os.RemoveAll(coreTarFilePath)
		return
	}

	// Decompress archived file to temp dir.
	tempDir := cnf.GetTempDir()
	os.RemoveAll(tempDir)
	if err := utils.Extract(coreTarFilePath, tempDir); err != nil {
		gprint.PrintError("Extract file failed: %+v.", err)
		os.RemoveAll(filepath.Dir(coreTarFilePath))
		return
	}

	// find dir to copy
	dirFinder := utils.NewFinder()
	dirFinder.SetFlags("core")
	dirFinder.SetFlagDirExcepted(false)
	dirFinder.Find(tempDir)
	dirToCopy := dirFinder.GetDirName()
	if dirToCopy == "" {
		gprint.PrintError("Can't find dir to copy.")
		os.RemoveAll(filepath.Dir(coreTarFilePath))
		return
	}

	// Copy extracted files to install directory.
	defer func() {
		os.RemoveAll(tempDir)
	}()
	if err := gutils.CopyDirectory(dirToCopy, moonbitInstallDir, true); err != nil {
		gprint.PrintError("Copy directory failed: %+v.", err)
		os.RemoveAll(filepath.Dir(coreTarFilePath))
		return
	}

	coreLibPath := filepath.Join(moonbitInstallDir, "core")
	moonBinaryPath := filepath.Join(moonbitInstallDir, "moon")
	if runtime.GOOS == gutils.Windows {
		moonBinaryPath += ".exe"
	}

	// bundle core lib.
	if ok, _ := gutils.PathIsExist(coreLibPath); ok {
		newPathEnv := utils.JoinPath(moonbitInstallDir, os.Getenv("PATH"))
		os.Setenv("PATH", newPathEnv)
		_, err := gutils.ExecuteSysCommand(
			false,
			moonbitInstallDir,
			moonBinaryPath,
			"bundle",
			"--all",
			"--source-dir",
			coreLibPath,
		)
		if err != nil {
			gprint.PrintError("Execute moon command failed: %+v.", err)
		}
	}
	os.RemoveAll(coreTarFilePath)
}
