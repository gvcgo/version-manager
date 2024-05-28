package cmds

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
)

const (
	// tui key envent name.
	KeyEventOpenHomePage     = "open-homepage"
	KeyEventSeachVersionList = "search-version-list"
)

/*
Show the SDK list supported by vmr.
TODO: use download.
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
	ll.SetRows(rows)
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
		HelpInfo: "open homepage",
	})

	// Search version list.
	ll.SetKeyEventForTable("s", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventSeachVersionList
			return tea.Quit
		},
		HelpInfo: "search versions for selected sdk",
	})

	// TODO: Show local installed.
}
