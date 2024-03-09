package use

import (
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/use/installer"
)

var TypstLspInstaller = &installer.Installer{
	AppName:        "typst-lsp",
	Version:        "0.12.1",
	Fetcher:        conf.GetFetcher(),
	IsZipFile:      false,
	BinaryRenameTo: "typst-lsp",
	FlagFileGetter: func() []string {
		r := []string{"typst-lsp"}
		if runtime.GOOS == gutils.Windows {
			r = []string{"typst-lsp.exe"}
		}
		return r
	},
	DUrlDecorator:      installer.DefaultDecorator,
	StoreMultiVersions: true,
	ForceReDownload:    true,
}

func TestTypstLsp() {
	zf := TypstLspInstaller.Download()
	TypstLspInstaller.Unzip(zf)
	TypstLspInstaller.Copy()
	TypstLspInstaller.CreateVersionSymbol()
	TypstLspInstaller.CreateBinarySymbol()
	TypstLspInstaller.SetEnv()
}
