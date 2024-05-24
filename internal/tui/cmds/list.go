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

/*
Show the SDK list supported by vmr.
*/

type SDKSha struct {
	Sha      string `json:"sha256"`
	HomePage string `json:"homepage"`
}

type SDKNameList map[string]SDKSha

type VMRSDKList struct {
	SdkList  SDKNameList
	Fetcher  *request.Fetcher
	selected string
}

func NewVMRSDKList() *VMRSDKList {
	return &VMRSDKList{
		SdkList: make(SDKNameList),
		Fetcher: request.NewFetcher(),
	}
}

func (v *VMRSDKList) ShowSDKList() (lastPressedKey string, selectedItem string) {
	dUrl := cnf.GetSDKListFileUrl()
	v.Fetcher.SetUrl(dUrl)
	v.Fetcher.Timeout = 10 * time.Second

	resp, _ := v.Fetcher.GetString()
	sdkList := make(SDKNameList)
	json.Unmarshal([]byte(resp), &sdkList)

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
	for k, v := range sdkList {
		rows = append(rows, table.Row{
			k,
			v.HomePage,
		})
	}
	SortVersionAscend(rows)
	ll.SetRows(rows)
	ll.Run()

	selectedItem = ll.GetSelected()
	lastPressedKey = ll.PressedKey
	return
}

func (v *VMRSDKList) RegisterKeyEvents(ll *table.List) {
	ll.SetKeyEventForTable("o", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			sr := l.Table.SelectedRow()
			if len(sr) > 1 {
				utils.OpenURL(sr[1])
			}
			l.PressedKey = "o"
			return nil
		},
		HelpInfo: "open homepage",
	})
	ll.SetKeyEventForTable("s", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.PressedKey = "s"
			return tea.Quit
		},
		HelpInfo: "search versions for selected sdk",
	})
}

func (v *VMRSDKList) GetSelected() string {
	return v.selected
}
