package use

import (
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var MavenInstaller = &installer.Installer{
	AppName:   "maven",
	Version:   "3.9.6",
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
	BinListGetter: func() []string {
		return []string{"mvn"}
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestMaven() {
	zf := MavenInstaller.Download()
	MavenInstaller.Unzip(zf)
	MavenInstaller.Copy()
	MavenInstaller.CreateVersionSymbol()
	MavenInstaller.CreateBinarySymbol()
	MavenInstaller.SetEnv()
}
