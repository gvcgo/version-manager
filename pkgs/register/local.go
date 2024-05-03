package register

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/atotto/clipboard"
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
	versionList := map[string]string{}

	currentVersion := filepath.Base(slink)
	coloredCurrentVersion := gprint.CyanStr(currentVersion) + gprint.YellowStr("<current>")
	versionList[coloredCurrentVersion] = currentVersion

	vList := []string{coloredCurrentVersion}
	dList, _ := os.ReadDir(vDir)
	for _, d := range dList {
		if d.IsDir() && d.Name() != currentVersion {
			coloredVersion := gprint.CyanStr(d.Name())
			versionList[coloredVersion] = d.Name()
			vList = append(vList, coloredVersion)
		}
	}

	columns := []gtable.Column{
		{Title: fmt.Sprintf("%s installed versions", appName), Width: 75},
	}

	rows := []gtable.Row{}

	for _, verName := range vList {
		rows = append(rows, gtable.Row{
			verName,
		})
	}

	t := gtable.NewTable(
		gtable.WithColumns(columns),
		gtable.WithRows(rows),
		gtable.WithFocused(true),
		gtable.WithHeight(15),
		gtable.WithWidth(160),
	)
	t.CopySelectedRow(true)
	t.Run()

	if coloredVersion, err := clipboard.ReadAll(); err == nil && coloredVersion != "" {
		// generate use command to clipboard.
		binPath, _ := os.Executable()
		binName := filepath.Base(binPath)
		if binName != "" {
			cmdStr := fmt.Sprintf(`%s use "%s@%s"`, binName, appName, versionList[coloredVersion])
			if err := clipboard.WriteAll(cmdStr); err == nil {
				gprint.PrintInfo("Now you can use 'ctrl+v/cmd+v' to swith to the selected for the selected SDK.")
			}
		}
	}
}
