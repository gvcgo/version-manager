package register

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/atotto/clipboard"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/gtable"
	"github.com/gvcgo/version-manager/pkgs/utils"
)

/*
Shows supported app list.
*/
func ShowAppList() {
	appList := []string{}
	for k := range VersionKeeper {
		appList = append(appList, k)
	}

	utils.SortVersions(appList)

	al := []string{}
	for i := len(appList) - 1; i >= 0; i-- {
		al = append(al, appList[i])
	}

	columns := []gtable.Column{
		{Title: "AppName", Width: 50},
		{Title: "Homepage", Width: 150},
	}

	rows := []gtable.Row{}

	for _, appName := range al {
		ver := VersionKeeper[appName]
		rows = append(rows, gtable.Row{
			appName,
			gprint.GreenStr(ver.GetHomepage()),
		})
	}

	t := gtable.NewTable(
		gtable.WithColumns(columns),
		gtable.WithRows(rows),
		gtable.WithFocused(true),
		gtable.WithHeight(15),
		gtable.WithWidth(150),
	)
	t.CopySelectedRow(true)
	t.Run()

	if appName, err := clipboard.ReadAll(); err == nil && appName != "" {
		// generate use command to clipboard.
		binPath, _ := os.Executable()
		binName := filepath.Base(binPath)
		if binName != "" {
			cmdStr := fmt.Sprintf("%s search %s", binName, appName)
			if err := clipboard.WriteAll(cmdStr); err == nil {
				gprint.PrintInfo("Now you can use 'ctrl+v/cmd+v' to search versions for the selected SDK.")
			}
		}
	}
}
