package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var FdInstaller = &installer.Installer{
	AppName:   "fd",
	Version:   "9.0.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"fd.1", "README.md"}
	},
	BinListGetter: func() []string {
		r := []string{"fd"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"fd.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestFd() {
	zf := FdInstaller.Download()
	FdInstaller.Unzip(zf)
	FdInstaller.Copy()
	FdInstaller.CreateVersionSymbol()
	FdInstaller.CreateBinarySymbol()
	FdInstaller.SetEnv()
}
