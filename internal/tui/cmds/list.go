package cmds

import (
	"encoding/json"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
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
*/

type SDKSha struct {
	Sha      string `json:"sha256"`
	HomePage string `json:"homepage"`
}

type SDKNameList map[string]SDKSha

type SDKSearcher struct {
	SdkList SDKNameList
	Fetcher *request.Fetcher
}

func NewSDKSearcher() *SDKSearcher {
	return &SDKSearcher{
		SdkList: make(SDKNameList),
		Fetcher: request.NewFetcher(),
	}
}

func (v *SDKSearcher) GetShaBySDKName(sdkName string) (ss SDKSha) {
	ss = v.SdkList[sdkName]
	return
}

func (v *SDKSearcher) Show() (nextEvent, selectedItem string) {
	dUrl := cnf.GetSDKListFileUrl()
	v.Fetcher.SetUrl(dUrl)
	v.Fetcher.Timeout = 10 * time.Second

	resp, _ := v.Fetcher.GetString()
	json.Unmarshal([]byte(resp), &v.SdkList)

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
	rows := []table.Row{}
	for k, v := range v.SdkList {
		rows = append(rows, table.Row{
			k,
			v.HomePage,
		})
	}
	SortVersionAscend(rows)
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
