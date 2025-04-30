package post

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/utils"
)

func init() {
	RegisterPostInstallHandler(RustSdkName, PostInstallForRust)
}

const (
	RustSdkName = "rust"
)

var (
	InstallRustupCmd []string = []string{
		"vmr",
		"use",
		"rustup@latest",
	}
	RustupDir = filepath.Join(cnf.GetVersionsDir(), "rustup_versions", "rustup-latest")
)

/*
post-installation handler for Rust.
*/
func PostInstallForRust(versionName string, version lua_global.Item) {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf("%s_versions", RustSdkName))
	rustInstallDir := filepath.Join(d, fmt.Sprintf("%s-%s", RustSdkName, versionName))
	binPath := filepath.Join(rustInstallDir, "Library", "bin")

	rustupInitPath := filepath.Join(RustupDir, "rustup-init")
	newName := "rustup"
	if runtime.GOOS == gutils.Windows {
		rustupInitPath = filepath.Join(RustupDir, "rustup-init.exe")
		newName = "rustup.exe"
	}

	homeDir, _ := os.UserHomeDir()

	if ok, _ := gutils.PathIsExist(rustupInitPath); !ok {
		task := utils.NewSysCommandRunner(
			true,
			homeDir,
			InstallRustupCmd...,
		)
		if err := task.Run(); err != nil {
			return
		}
	}

	// install rustup to cargo bin path.
	if _, err := gutils.CopyFile(rustupInitPath, filepath.Join(binPath, newName)); err == nil && runtime.GOOS != gutils.Windows {
		gutils.ExecuteSysCommand(
			true,
			homeDir,
			"chmox",
			"+x",
			filepath.Join(binPath, newName),
		)
	}
}
