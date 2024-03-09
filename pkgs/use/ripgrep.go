package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var RipgrepInstaller = &installer.Installer{
	AppName:   "ripgrep",
	Version:   "14.1.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"rg"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"rg.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"rg"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"rg.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestRipgrep() {
	zf := RipgrepInstaller.Download()
	RipgrepInstaller.Unzip(zf)
	RipgrepInstaller.Copy()
	RipgrepInstaller.CreateVersionSymbol()
	RipgrepInstaller.CreateBinarySymbol()
	RipgrepInstaller.SetEnv()
}
