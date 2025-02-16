package cliui

import (
	"fmt"
	"os"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/installer/install"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
)

type SDKSearcher struct {
	Plugins *plugin.Plugins
}

func NewSDKSearcher() *SDKSearcher {
	return &SDKSearcher{
		Plugins: plugin.NewPlugins(),
	}
}

func (v *SDKSearcher) Show() (nextEvent, selectedItem string) {
	ll := table.NewList()
	ll.SetListType(table.SDKList)
	v.RegisterKeyEvents(ll)

	_, w, _ := terminal.GetTerminalSize()
	if w > 30 {
		w -= 30
	} else {
		w = 120
	}
	ll.SetHeader([]table.Column{
		{Title: "sdkname", Width: 20},
		{Title: "homepage", Width: w},
	})
	rows := v.Plugins.GetPluginSortedRows()
	if len(rows) == 0 {
		gprint.PrintWarning("No sdk found!")
		gprint.PrintWarning("Please check if you have a proxy or reverse proxy available.")
		return
	}
	ll.SetRows(rows)
	ll.Run()

	selectedItem = ll.GetSelected()
	nextEvent = ll.NextEvent
	return
}

func (v *SDKSearcher) ShowInstalledOnly() (nextEvent, selectedItem string) {
	ll := table.NewList()
	ll.SetListType(table.SDKList)
	v.RegisterKeyEvents(ll)

	_, w, _ := terminal.GetTerminalSize()
	if w > 30 {
		w -= 30
	} else {
		w = 120
	}
	ll.SetHeader([]table.Column{
		{Title: "installed sdk", Width: 20},
		{Title: "homepage", Width: w},
	})

	rows := v.Plugins.GetPluginSortedRows()
	installedRows := []table.Row{}
	for _, r := range rows {
		if install.IsSDKInstalledByVMR(r[0]) {
			installedRows = append(installedRows, r)
		}
	}
	if len(installedRows) == 0 {
		gprint.PrintWarning("no installed sdk found!")
		return
	}
	ll.SetRows(installedRows)
	ll.Run()

	selectedItem = ll.GetSelected()
	nextEvent = ll.NextEvent
	return
}

func (v *SDKSearcher) PrintInstalledSDKs() {
	sdkList := v.GetInstalledSDKList()
	for _, sdkName := range sdkList {
		fmt.Println(gprint.CyanStr("%s", sdkName))
	}
}

func (v *SDKSearcher) GetInstalledSDKList() (sdkList []string) {
	rows := v.Plugins.GetPluginSortedRows()
	installedRows := []table.Row{}
	for _, r := range rows {
		if install.IsSDKInstalledByVMR(r[0]) {
			installedRows = append(installedRows, r)
		}
	}
	if len(installedRows) == 0 {
		gprint.PrintWarning("no installed sdk found!")
		return
	}

	for _, r := range installedRows {
		sdkList = append(sdkList, r[0])
	}
	sdkList = v.GetSDKInstalledByCondaForge(sdkList)
	return
}

// SDK supported by Conda but not by VMR.
func (v *SDKSearcher) GetSDKInstalledByCondaForge(sdkInstalledByVMR []string) []string {
	dedup := map[string]struct{}{}
	for _, sdkName := range sdkInstalledByVMR {
		dedup[sdkName] = struct{}{}
	}
	dirs, _ := os.ReadDir(cnf.GetVersionsDir())
	for _, d := range dirs {
		sdkName := strings.TrimSuffix(d.Name(), install.VersionDirSuffix)
		if _, ok := dedup[sdkName]; ok {
			continue
		}
		sdkInstalledByVMR = append(sdkInstalledByVMR, sdkName)
	}
	return sdkInstalledByVMR
}

func (v *SDKSearcher) RegisterKeyEvents(ll *table.List) {
	// Open homepage.
	ll.SetKeyEventForTable("o", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			sr := l.Table.SelectedRow()
			if len(sr) > 1 {
				utils.OpenURL(sr[1])
			}
			l.NextEvent = ""
			return nil
		},
		HelpInfo: "open homepage of the selected sdk",
	})
}
