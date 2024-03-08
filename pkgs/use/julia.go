package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

// TODO: copy symbolic file failed.
var JuliaInstaller = &installer.Installer{
	AppName:   "julia",
	Version:   "1.10.2",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE.md"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"julia"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"julia.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestJulia() {
	zf := JuliaInstaller.Download()
	JuliaInstaller.Unzip(zf)
	JuliaInstaller.Copy()
	JuliaInstaller.CreateVersionSymbol()
	JuliaInstaller.CreateBinarySymbol()
	JuliaInstaller.SetEnv()
}
