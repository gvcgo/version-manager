package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var DenoInstaller = &installer.Installer{
	AppName:   "deno",
	Version:   "1.41.1",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"deno"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"deno.exe"}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"deno"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"deno.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestDeno() {
	zf := DenoInstaller.Download()
	DenoInstaller.Unzip(zf)
	DenoInstaller.Copy()
	DenoInstaller.CreateVersionSymbol()
	DenoInstaller.CreateBinarySymbol()
	DenoInstaller.SetEnv()
}
