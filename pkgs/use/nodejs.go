package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var NodejsInstaller = &installer.Installer{
	AppName:   "nodejs",
	Version:   "20.11.1",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"LICENSE", "README.md"}
	},
	BinDirGetter: func(version string) [][]string {
		r := [][]string{{"bin"}}
		if runtime.GOOS == gutils.Windows {
			r = [][]string{}
		}
		return r
	},
	BinListGetter: func() []string {
		r := []string{"node", "npm", "npx", "corepack"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"node.exe", "npm.cmd", "npx.cm", "corepack.cmd"}
		}
		return r
	},
	StoreMultiVersions: true,
}

func TestNodejs() {
	zf := NodejsInstaller.Download()
	NodejsInstaller.Unzip(zf)
	NodejsInstaller.Copy()
	NodejsInstaller.CreateVersionSymbol()
	NodejsInstaller.CreateBinarySymbol()
	NodejsInstaller.SetEnv()
}
