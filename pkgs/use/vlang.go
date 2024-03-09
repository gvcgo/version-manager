package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var VlangInstaller = &installer.Installer{
	AppName:   "v",
	Version:   "0.4.4",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"cmd", "v"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"cmd", "v.exe"}
		}
		return r
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{},
			{"cmd", "tools"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"v", "vdoctor", "vup"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"v.exe", "vdoctor.exe", "vup.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestVlang() {
	zf := VlangInstaller.Download()
	VlangInstaller.Unzip(zf)
	VlangInstaller.Copy()
	VlangInstaller.CreateVersionSymbol()
	VlangInstaller.CreateBinarySymbol()
	VlangInstaller.SetEnv()
}
