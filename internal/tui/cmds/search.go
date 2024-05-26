package cmds

import (
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/download"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
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

func (s *VersionSearcher) Search(sdkName, newSha256 string) {
	s.filteredVersions = download.GetVersionList(sdkName, newSha256)
	if s.ToShowList {
		s.Show()
	}
}

func (s *VersionSearcher) Show() (nextEvent, selectedItem string) {
	if len(s.filteredVersions) == 0 {
		gprint.PrintInfo("No versions found for current platform.")
		return
	}
	ll := table.NewList()
	ll.SetListType(table.SDKList)
	// s.RegisterKeyEvents(ll)

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
	rows := []table.Row{}
	for vName, vItem := range s.filteredVersions {
		rows = append(rows, table.Row{
			vName,
			vItem.Installer,
		})
	}
	SortVersions(rows)
	ll.SetRows(rows)
	ll.Run()

	selectedItem = ll.GetSelected()
	nextEvent = ll.NextEvent
	return
}

// TODO: install, switch-to, session-only, lock-version
func (s *VersionSearcher) RegisterKeyEvents(ll *table.List) {
}
