package search

import (
	"strings"

	"github.com/gvcgo/version-manager/pkgs/tui"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

/*
Show available versions for an app.
*/
type Searcher struct {
	VersionInfo *versions.VersionInfo
}

func NewSearcher() (s *Searcher) {
	s = &Searcher{}
	return
}

func (s *Searcher) Search(appName string) {
	s.VersionInfo = versions.NewVInfo(appName)
	s.VersionInfo.RegisterArchHandler(versions.ArchHandlerList[appName])
	s.VersionInfo.RegisterOsHandler(versions.OsHandlerList[appName])
	vl := s.VersionInfo.GetSortedVersionList()
	tui.ShowAsPortView(appName, strings.Join(vl, "\n"))
}
