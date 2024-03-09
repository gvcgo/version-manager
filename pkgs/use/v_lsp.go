package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var VlangLspInstaller = &installer.Installer{
	AppName:   "v-analyzer",
	Version:   "0.0.3",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"v-analyzer"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"v-analyzer.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestVlangLsp() {
	zf := VlangLspInstaller.Download()
	VlangLspInstaller.Unzip(zf)
	VlangLspInstaller.Copy()
	VlangLspInstaller.CreateVersionSymbol()
	VlangLspInstaller.CreateBinarySymbol()
	VlangLspInstaller.SetEnv()
}
