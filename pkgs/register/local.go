package register

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/gtable"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
	"github.com/gvcgo/version-manager/pkgs/installer"
)

/*
Shows installed versions.
*/
var AndroidSDKNames []string = []string{
	"build-tools",
	"platforms",
	"platform-tools",
	"system-images",
	"ndks",
	"add-ons",
	"extras",
}

func ShowInstalledAndroidSDK(appName string) {
	for _, name := range AndroidSDKNames {
		if appName == name {
			searcher := installer.NewSDKManagerSearcher()
			searcher.ShowInstalledPackages()
			os.Exit(0)
		}
	}
}

func ShowInstalled(appName string) {
	// Try to show installed android SDKs.
	ShowInstalledAndroidSDK(appName)

	vDir := conf.GetVMVersionsDir(appName)
	slink, _ := os.Readlink(filepath.Join(vDir, appName))
	if ok, _ := gutils.PathIsExist(slink); !ok {
		gprint.PrintInfo("No versions installed for %s.", appName)
		return
	}
	currentVersion := filepath.Base(slink)
	vList := []string{gprint.CyanStr(currentVersion) + gprint.YellowStr("<current>")}
	dList, _ := os.ReadDir(vDir)
	for _, d := range dList {
		if d.IsDir() && d.Name() != currentVersion {
			vList = append(vList, gprint.CyanStr(d.Name()))
		}
	}

	columns := []gtable.Column{
		{Title: fmt.Sprintf("%s installed versions", appName), Width: 75},
	}

	rows := []gtable.Row{}

	for _, verName := range vList {
		rows = append(rows, gtable.Row{
			gprint.CyanStr(verName),
		})
	}

	t := gtable.NewTable(
		gtable.WithColumns(columns),
		gtable.WithRows(rows),
		gtable.WithFocused(true),
		gtable.WithHeight(15),
		gtable.WithWidth(160),
	)
	t.Run()
}
