package use

import (
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var BunInstaller = &installer.Installer{
	AppName:   "bun",
	Version:   "1.0.9",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bun"}
	},
	BinListGetter: func() []string {
		return []string{"bun"}
	},
	StoreMultiVersions: true,
}

func TestBun() {
	zf := BunInstaller.Download()
	BunInstaller.Unzip(zf)
	BunInstaller.Copy()
	BunInstaller.CreateVersionSymbol()
	BunInstaller.CreateBinarySymbol()
	BunInstaller.SetEnv()
}
