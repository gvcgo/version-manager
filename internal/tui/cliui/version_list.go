package cliui

import (
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/installer"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

type VersionSearcher struct {
	SDKName          string
	ToShowList       bool
	filteredVersions map[string]lua_global.Item
	ToSearchByConda  bool
}

func NewVersionSearcher() (sv *VersionSearcher) {
	sv = &VersionSearcher{
		ToShowList:       true,
		filteredVersions: make(map[string]lua_global.Item),
		ToSearchByConda:  false,
	}
	return
}

func (s *VersionSearcher) EnableCondaSearch() {
	s.ToSearchByConda = true
}

func (s *VersionSearcher) GetVersionByVersionName(vName string) (item lua_global.Item) {
	item = s.filteredVersions[vName]
	return
}

func (s *VersionSearcher) Search(sdkName, newSha256 string) (nextEvent, selectedItem string) {
	s.SDKName = sdkName
	if !s.ToSearchByConda {
		pls := plugin.NewPlugins()
		pls.LoadAll()
		p := pls.GetPluginBySDKName(sdkName)
		versions := plugin.NewVersions(p.PluginName)
		if versions != nil {
			s.filteredVersions = versions.GetSdkVersions()
		}
	} else {
		condaSearcher := installer.NewCondaSearcher(sdkName)
		s.filteredVersions = condaSearcher.GetVersions()
	}
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

	pls := plugin.NewPlugins()
	pls.LoadAll()
	p := pls.GetPluginBySDKName(s.SDKName)
	versions := plugin.NewVersions(p.PluginName)
	defer versions.CloseLua()
	if versions == nil {
		return
	}
	rows := versions.GetSortedVersionList()

	if len(rows) == 0 {
		gprint.PrintWarning("No versions found for current platform.")
		return
	}
	newRows := []table.Row{}
	// filter invalid version name.
	for _, row := range rows {
		if len(row[0]) == 0 {
			continue
		}
		newRows = append(newRows, row)
	}
	ll.SetRows(newRows)
	ll.Run()

	selectedItem = strings.TrimSuffix(ll.GetSelected(), "-lts")
	nextEvent = ll.NextEvent
	return
}

func (s *VersionSearcher) RegisterKeyEvents(ll *table.List) {
}
