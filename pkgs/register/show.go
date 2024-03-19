package register

import (
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/gtable"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

/*
Shows supported app list.
*/
func ShowAppList() {

	appList := []string{}
	for k := range VersionKeeper {
		appList = append(appList, k)
	}
	al := versions.SortStringList(appList)
	// content := strings.Join(al, "\n")
	// tui.ShowAsPortView("supported apps", content)

	columns := []gtable.Column{
		{Title: "AppName", Width: 50},
		{Title: "Homepage", Width: 150},
	}

	rows := []gtable.Row{}

	for _, appName := range al {
		ver := VersionKeeper[appName]
		rows = append(rows, gtable.Row{
			gprint.CyanStr(appName),
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
	t.Run()
}
