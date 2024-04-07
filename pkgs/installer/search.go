package installer

import (
	"fmt"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/gtable"
	"github.com/gvcgo/version-manager/pkgs/utils"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

type Searcher struct {
	VersionInfo *versions.VersionInfo
}

func NewSearcher() (s *Searcher) {
	s = &Searcher{}
	return
}

func (s *Searcher) init(appName string) {
	s.VersionInfo = versions.NewVInfo(appName)
	s.VersionInfo.RegisterArchHandler(versions.ArchHandlerList[appName])
	s.VersionInfo.RegisterOsHandler(versions.OsHandlerList[appName])
}

// Gets version list.
func (s *Searcher) GetVersions(appName string) map[string]versions.VersionList {
	s.init(appName)
	return s.VersionInfo.GetVersions()
}

// Shows version list.
func (s *Searcher) Search(appName string) {
	if appName == "cmdtools" {
		s.init("sdkmanager")
	} else {
		s.init(appName)
	}
	vl := s.VersionInfo.GetSortedVersionList()
	if len(vl) == 0 {
		gprint.PrintWarning("No versions found!")
		return
	}

	// Sort versions.
	utils.SortVersions(vl)

	columns := []gtable.Column{
		{Title: gprint.CyanStr(fmt.Sprintf("%v available versions", appName)), Width: 150},
	}

	rows := []gtable.Row{}

	for _, verName := range vl {
		rows = append(rows, gtable.Row{
			verName,
		})
	}

	t := gtable.NewTable(
		gtable.WithColumns(columns),
		gtable.WithRows(rows),
		gtable.WithFocused(true),
		gtable.WithHeight(25),
		gtable.WithWidth(100),
	)
	t.CopySelectedRow(true)
	t.Run()
}
