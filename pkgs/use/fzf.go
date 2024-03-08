package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var FzFInstaller = installer.Installer{
	AppName:   "fzf",
	Version:   "0.46.1",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"fzf"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"fzf.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"fzf"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"fzf.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestFzF() {
	zf := FzFInstaller.Download()
	FzFInstaller.Unzip(zf)
	FzFInstaller.Copy()
	FzFInstaller.CreateVersionSymbol()
	FzFInstaller.CreateBinarySymbol()
	FzFInstaller.SetEnv()
}
