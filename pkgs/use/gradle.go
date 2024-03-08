package use

import (
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var GradleInstaller = &installer.Installer{
	AppName:   "gradle",
	Version:   "8.6",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	// DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestGradle() {
	zf := GradleInstaller.Download()
	GradleInstaller.Unzip(zf)
	GradleInstaller.Copy()
	GradleInstaller.CreateVersionSymbol()
	GradleInstaller.CreateBinarySymbol()
	GradleInstaller.SetEnv()
}
