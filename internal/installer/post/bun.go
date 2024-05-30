package post

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
)

func init() {
	RegisterPostInstallHandler(BunSdkName, PostInstallForBun)
}

const (
	BunSdkName string = "bun"
)

/*
post-installation handler for Bun.
*/
func PostInstallForBun(versionName string, version download.Item) {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf("%s_versions", BunSdkName))
	bunInstallDir := filepath.Join(d, fmt.Sprintf("%s-%s", BunSdkName, versionName))
	binPath := filepath.Join(bunInstallDir, "bun")
	if runtime.GOOS == "windows" {
		binPath = filepath.Join(bunInstallDir, "bun.exe")
	}
	if ok, _ := gutils.PathIsExist(binPath); !ok {
		return
	}
	symbolPath := filepath.Join(bunInstallDir, "bunx")
	if runtime.GOOS == "windows" {
		symbolPath = filepath.Join(bunInstallDir, "bunx.exe")
		os.Link(binPath, symbolPath)
		return
	}
	os.Symlink(binPath, symbolPath)
}
