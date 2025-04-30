package cmds

import (
	"fmt"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/installer"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

const (
	KeyEventClearCachedFileForAVersion = "clear-cached-file-for-a-version"
	KeyEventRemoveAnInstalledVersion   = "remove-an-installed-version"
)

type LocalInstalled struct {
	finder         *installer.InstalledVersionFinder
	PluginName     string
	CurrentVersion string
	VersionList    []string
}

func NewLocalInstalled() (l *LocalInstalled) {
	l = &LocalInstalled{}
	return
}

func (l *LocalInstalled) Search(pluginName string) {
	l.PluginName = pluginName
	l.finder = installer.NewIVFinder(pluginName)
	l.VersionList, l.CurrentVersion = l.finder.FindAll()
}

func (l *LocalInstalled) Show() (nextEvent, selectedItem string) {

	ll := table.NewList()
	ll.SetListType(table.VersionList)
	l.RegisterKeyEvents(ll)

	_, w, _ := terminal.GetTerminalSize()
	if w > 30 {
		w -= 30
	} else {
		w = 120
	}
	ll.SetHeader([]table.Column{
		{Title: fmt.Sprintf("%s installed versions", l.PluginName), Width: 80},
	})

	rows := []table.Row{}
	for _, v := range l.VersionList {
		if v == l.CurrentVersion {
			v = v + "<current>"
		}
		rows = append(rows, table.Row{
			v,
		})
	}
	if len(rows) == 0 {
		gprint.PrintInfo("No versions found for %s", l.PluginName)
		return
	}
	ll.SetRows(rows)
	ll.Run()

	selectedItem = strings.TrimSuffix(ll.GetSelected(), "<current>")
	nextEvent = ll.NextEvent
	return
}

func (l *LocalInstalled) RegisterKeyEvents(ll *table.List) {
	ll.SetKeyEventForTable("c", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventClearCachedFileForAVersion
			return tea.Quit
		},
		HelpInfo: "clear cached file for the selected version",
	})

	ll.SetKeyEventForTable("r", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventRemoveAnInstalledVersion
			return tea.Quit
		},
		HelpInfo: "remove the selected version",
	})

	ll.SetKeyEventForTable("b", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventBacktoPreviousPage
			return tea.Quit
		},
		HelpInfo: "back to previous page",
	})

	ll.SetKeyEventForTable("l", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventLockVersion
			return tea.Quit
		},
		HelpInfo: "lock the selected version for curret project",
	})

	ll.SetKeyEventForTable("u", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventUseVersionGlobally
			return tea.Quit
		},
		HelpInfo: "switch to the selected version globally",
	})

	ll.SetKeyEventForTable("s", table.KeyEvent{
		Event: func(key string, l *table.List) tea.Cmd {
			l.NextEvent = KeyEventUseVersionSessionly
			return tea.Quit
		},
		HelpInfo: "switch to the selected version only in current session",
	})
}
