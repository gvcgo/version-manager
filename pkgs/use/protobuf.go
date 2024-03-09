package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var ProtobufInstaller = &installer.Installer{
	AppName:   "protobuf",
	Version:   "25.3",
	Fetcher:   conf.GetFetcher(),
	IsZipFile: true,
	FlagFileGetter: func() []string {
		return []string{"bin"}
	},
	BinDirGetter: func(version string) [][]string {
		return [][]string{
			{"bin"},
		}
	},
	BinListGetter: func() []string {
		r := []string{"protoc"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"protoc.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
}

func TestProtobuf() {
	zf := ProtobufInstaller.Download()
	ProtobufInstaller.Unzip(zf)
	ProtobufInstaller.Copy()
	ProtobufInstaller.CreateVersionSymbol()
	ProtobufInstaller.CreateBinarySymbol()
	ProtobufInstaller.SetEnv()
}
