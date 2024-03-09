package use

import (
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var ZigLspInstaller = &installer.Installer{
	AppName:   "zls",
	Version:   "0.11.0",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"README.md"}
	},
	BinDirGetter: func(version string) [][]string {
		if strings.HasPrefix(version, "0.1.") || strings.HasPrefix(version, "0.2.") {
			return [][]string{}
		}
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"zls"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"zls.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestZigLsp() {
	zf := ZigLspInstaller.Download()
	ZigLspInstaller.Unzip(zf)
	ZigLspInstaller.Copy()
	ZigLspInstaller.CreateVersionSymbol()
	ZigLspInstaller.CreateBinarySymbol()
	ZigLspInstaller.SetEnv()
}
