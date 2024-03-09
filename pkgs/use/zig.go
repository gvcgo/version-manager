package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var ZigInstaller = &installer.Installer{
	AppName:   "zig",
	Version:   "0.11.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE"}
	},
	BinListGetter: func() []string {
		r := []string{"zig"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"zig.exe"}
		}
		return r
	},
	StoreMultiVersions: true,
}

func TestZig() {
	zf := ZigInstaller.Download()
	ZigInstaller.Unzip(zf)
	ZigInstaller.Copy()
	ZigInstaller.CreateVersionSymbol()
	ZigInstaller.CreateBinarySymbol()
	ZigInstaller.SetEnv()
}
