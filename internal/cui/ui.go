package cui

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/version-manager/internal/cui/column"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
)

type CurrentView int

const (
	LeftColumn CurrentView = iota
	RightColumn
	Prompt
)

type UI struct {
	prompt   tea.Model
	left     *column.Column
	right    *column.Column
	previous CurrentView
	current  CurrentView
	plugins  *plugin.Plugins
}
