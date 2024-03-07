package use

import (
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var FlutterInstaller = &installer.Installer{
	AppName:   "flutter",
	Version:   "3.19.2",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"README.md", "LICENSE", "CODEOWNERS"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"dart", "flutter"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"dart.bat", "flutter.bat"}
		}
		return r
	},
	DUrlDecorator: func(dUrl string, ft *request.Fetcher) string {
		return strings.ReplaceAll(dUrl, "https://storage.googleapis.com", "https://storage.flutter-io.cn")
	},
	StoreMultiVersions: true,
}

func TestFlutter() {
	zf := FlutterInstaller.Download()
	FlutterInstaller.Unzip(zf)
	FlutterInstaller.Copy()
	FlutterInstaller.CreateVersionSymbol()
	FlutterInstaller.CreateBinarySymbol()
	FlutterInstaller.SetEnv()
}
