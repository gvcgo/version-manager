package cmds

import (
	"github.com/gvcgo/version-manager/internal/terminal"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

func GetTableHeader(sdkTitle string) []table.Column {
	_, w, _ := terminal.GetTerminalSize()
	if w > 60 {
		w -= 60
	} else {
		w = 120
	}
	t := []table.Column{
		{Title: "plugin_name", Width: 20},
		{Title: "plugin_version", Width: 20},
		{Title: sdkTitle, Width: 20},
		{Title: "homepage", Width: w},
	}
	return t
}
