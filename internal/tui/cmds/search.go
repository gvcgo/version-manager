package cmds

import (
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

const (
	KeyEventUseVersionGlobally  string = "use-version-globally"
	KeyEventUseVersionSessionly string = "use-version-sessionly"
	KeyEventLockVersion         string = "lock-version"
)

/*
Search version list for SDK.
*/

type VersionSearcher struct {
	ToShowList bool
	pluginName string
	versions   *plugin.Versions
	vList      map[string]lua_global.Item
}

func NewVersionSearcher() (sv *VersionSearcher) {
	sv = &VersionSearcher{
		ToShowList: true,
		vList:      make(map[string]lua_global.Item),
	}
	return
}

func (s *VersionSearcher) GetVersionByVersionName(vName string) (item lua_global.Item) {
	if s.versions == nil {
		return
	}

	if len(s.vList) == 0 {
		return
	}
	item = s.vList[vName]
	return
}

func (s *VersionSearcher) Search(pluginName string) (nextEvent, selectedItem string) {
	s.pluginName = pluginName
	s.versions = plugin.NewVersions(pluginName)

	s.vList = s.versions.GetSdkVersions()

	if s.ToShowList {
		nextEvent, selectedItem = s.Show()
	}
	return
}

func (s *VersionSearcher) Show() (nextEvent, selectedItem string) {
	if len(s.vList) == 0 || s.versions == nil {
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
		{Title: s.pluginName, Width: 20},
		{Title: "installer", Width: w},
	})
	rows := s.versions.GetSortedVersionList()
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
	ll.SetKeyEventForTable("i", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventUseVersionGlobally
			return tea.Quit
		},
		HelpInfo: "install selected version globally",
	})

	ll.SetKeyEventForTable("s", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventUseVersionSessionly
			return tea.Quit
		},
		HelpInfo: "use selected version only in current session",
	})

	ll.SetKeyEventForTable("l", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventLockVersion
			return tea.Quit
		},
		HelpInfo: "lock selected version for current project",
	})

	ll.SetKeyEventForTable("b", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventBacktoPreviousPage
			return tea.Quit
		},
		HelpInfo: "back to previous page",
	})
}
