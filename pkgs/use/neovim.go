package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var NeovimInstaller = &installer.Installer{
	AppName:   "neovim",
	Version:   "0.9.5",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"nvim"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"nvim.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestNeovim() {
	zf := NeovimInstaller.Download()
	NeovimInstaller.Unzip(zf)
	NeovimInstaller.Copy()
	NeovimInstaller.CreateVersionSymbol()
	NeovimInstaller.CreateBinarySymbol()
	NeovimInstaller.SetEnv()
}
