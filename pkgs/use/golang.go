package use

import (
	"path/filepath"

	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var GoInstaller = &installer.Installer{
	AppName:   "go",
	Version:   "1.22.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	FlagFileGetter: func() []string {
		return []string{"VERSION", "LICENSE"}
	},
	EnvGetter: func(appName, version string) []installer.Env {
		return []installer.Env{
			{Name: "GOROOT", Value: filepath.Join(conf.GetVMVersionsDir(appName), appName)},
		}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestGo() {
	zf := GoInstaller.Download()
	GoInstaller.Unzip(zf)
	GoInstaller.Copy()
	GoInstaller.CreateVersionSymbol()
	GoInstaller.CreateBinarySymbol()
	GoInstaller.SetEnv()
}
