package search

import (
	"strings"

	"github.com/gvcgo/version-manager/pkgs/tui"
	"github.com/gvcgo/version-manager/pkgs/versions"
)

/*
Show app list.
*/
func ShowAppList() {
	content := strings.Join(versions.AppList, "\n")
	tui.ShowAsPortView("supported apps", content)
}
