package use

import (
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var KotlinInstaller = &installer.Installer{
	AppName:   "kotlin",
	Version:   "1.9.23",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin", "tools", "klib"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	AddBinDirToPath:    true,
}

func TestKotlin() {
	zf := KotlinInstaller.Download()
	KotlinInstaller.Unzip(zf)
	KotlinInstaller.Copy()
	KotlinInstaller.CreateVersionSymbol()
	KotlinInstaller.CreateBinarySymbol()
	KotlinInstaller.SetEnv()
}
