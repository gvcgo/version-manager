package cliui

import (
	"fmt"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/version-manager/internal/installer"
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

type LocalInstalled struct {
	finder         *installer.InstalledVersionFinder
	SDKName        string
	CurrentVersion string
	VersionList    []string
}

func NewLocalInstalled() (l *LocalInstalled) {
	l = &LocalInstalled{}
	return
}

func (l *LocalInstalled) Search(sdkName string) {
	l.SDKName = sdkName
	l.finder = installer.NewIVFinder(sdkName)
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
		{Title: fmt.Sprintf("%s installed versions", l.SDKName), Width: 80},
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
		gprint.PrintInfo("No versions found for %s", l.SDKName)
		return
	}
	ll.SetRows(rows)
	ll.Run()

	selectedItem = strings.TrimSuffix(ll.GetSelected(), "<current>")
	nextEvent = ll.NextEvent
	return
}

func (l *LocalInstalled) RegisterKeyEvents(ll *table.List) {
}
