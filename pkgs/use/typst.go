package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var TypstInstaller = &installer.Installer{
	AppName:   "typst",
	Version:   "0.10.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		r := []string{"LICENSE"}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"typst"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"typst.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
}

func TestTypst() {
	zf := TypstInstaller.Download()
	TypstInstaller.Unzip(zf)
	TypstInstaller.Copy()
	TypstInstaller.CreateVersionSymbol()
	TypstInstaller.CreateBinarySymbol()
	TypstInstaller.SetEnv()
}
