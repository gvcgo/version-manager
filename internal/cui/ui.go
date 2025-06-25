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

type PromptView struct {
	pre     *PromptView
	model   tea.Model
	handler func(string) error
	next    *PromptView
}

type UI struct {
	prompt      tea.Model
	left        *column.Column
	right       *column.Column
	promptStack *PromptView
	previous    CurrentView
	current     CurrentView
	plugins     *plugin.Plugins
}
