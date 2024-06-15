package cmds

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/installer/install"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	// tui key envent name.
	KeyEventOpenHomePage         = "open-homepage"
	KeyEventSeachVersionList     = "search-version-list"
	KeyEventShowLocalInstalled   = "show-installed-versions"
	KeyEventRemoveLocalInstalled = "remove-installed-versions"
	KeyEventClearLocalCached     = "clear-local-cached-files"
	KeyEventBacktoPreviousPage   = "back-to-previous-page"
	KeyEventWhatsInstalled       = "show-installed-sdks"
)

/*
Show the SDK list supported by vmr.
*/

type SDKSearcher struct {
	SdkList download.SDKList
}

func NewSDKSearcher() *SDKSearcher {
	return &SDKSearcher{
		SdkList: make(download.SDKList),
	}
}

func (v *SDKSearcher) GetShaBySDKName(sdkName string) (ss string) {
	if s, ok := v.SdkList[sdkName]; ok {
		ss = s.Sha256
	}
	return
}

func (v *SDKSearcher) GetSDKItemByName(sdkName string) (item download.SDK) {
	item = v.SdkList[sdkName]
	return
}

func (v *SDKSearcher) Show() (nextEvent, selectedItem string) {
	v.SdkList = download.GetSDKList()

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
	rows := download.GetSDKSortedRows(v.SdkList)
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
	rows := download.GetSDKSortedRows(v.SdkList)

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

func (v *SDKSearcher) RegisterKeyEvents(ll *table.List) {
	// Open homepage.
	ll.SetKeyEventForTable("o", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			sr := l.Table.SelectedRow()
			if len(sr) > 1 {
				utils.OpenURL(sr[1])
			}
			l.NextEvent = KeyEventOpenHomePage
			return nil
		},
		HelpInfo: "open homepage of the selected sdk",
	})

	// Search version list.
	ll.SetKeyEventForTable("s", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventSeachVersionList
			return tea.Quit
		},
		HelpInfo: "search available versions for selected sdk",
	})

	ll.SetKeyEventForTable("l", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventShowLocalInstalled
			return tea.Quit
		},
		HelpInfo: "show local installed versions of the selected sdk",
	})

	ll.SetKeyEventForTable("r", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventRemoveLocalInstalled
			return tea.Quit
		},
		HelpInfo: "remove all local installed versions of the selected sdk",
	})

	ll.SetKeyEventForTable("c", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventClearLocalCached
			return tea.Quit
		},
		HelpInfo: "clear all local cached files of the selected sdk",
	})
	ll.SetKeyEventForTable("w", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventWhatsInstalled
			return tea.Quit
		},
		HelpInfo: "show the list of sdks installed by VMR on your system",
	})
}
