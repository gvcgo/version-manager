package cmds

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

const (
	KeyEventInstallGlobally     string = "install-globally"
	KeyEventUseVersionGlobally  string = "use-version-globally"
	KeyEventUseVersionSessionly string = "use-version-sessionly"
	KeyEventLockVersion         string = "lock-version"
)

/*
Search version list for SDK.
TODO: use download.
*/

type VersionSearcher struct {
	SDKName          string
	Fetcher          *request.Fetcher
	ToShowList       bool
	filteredVersions map[string]download.Item
}

func NewVersionSearcher() (sv *VersionSearcher) {
	sv = &VersionSearcher{
		Fetcher:          request.NewFetcher(),
		ToShowList:       true,
		filteredVersions: make(map[string]download.Item),
	}
	return
}

func (s *VersionSearcher) GetVersionByVersionName(vName string) (item download.Item) {
	item = s.filteredVersions[vName]
	return
}

func (s *VersionSearcher) Search(sdkName, newSha256 string) (nextEvent, selectedItem string) {
	s.filteredVersions = download.GetVersionList(sdkName, newSha256)
	if s.ToShowList {
		nextEvent, selectedItem = s.Show()
	}
	return
}

func (s *VersionSearcher) Show() (nextEvent, selectedItem string) {
	if len(s.filteredVersions) == 0 {
		gprint.PrintInfo("No versions found for current platform.")
		return
	}
	ll := table.NewList()
	ll.SetListType(table.SDKList)
	s.RegisterKeyEvents(ll)

	_, w, _ := terminal.GetTerminalSize()
	if w > 30 {
		w -= 30
	} else {
		w = 120
	}
	ll.SetHeader([]table.Column{
		{Title: s.SDKName, Width: 20},
		{Title: "installer", Width: w},
	})
	rows := download.GetVersionsSortedRows(s.filteredVersions)
	ll.SetRows(rows)
	ll.Run()

	selectedItem = ll.GetSelected()
	nextEvent = ll.NextEvent
	return
}

// TODO: install, switch-to, session-only, lock-version
func (s *VersionSearcher) RegisterKeyEvents(ll *table.List) {
	ll.SetKeyEventForTable("i", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventInstallGlobally
			return tea.Quit
		},
	})
}
