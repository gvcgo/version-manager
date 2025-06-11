package types

import (
	"fmt"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/lipgloss"
)

const (
	HelpInfoPattern string = "<%s> %s"
)

var (
	FocusedStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#00AF00"))
	BluredStyle  = lipgloss.NewStyle().Foreground(lipgloss.Color("#B0B0B0"))
	HelpStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("#FF8C00"))
)

type Hook func() error

type IKeyMap interface {
	GetHelpInfo() string
}

type CommonKeyMap struct {
	Quit key.Binding
}

func GetCommonKeyMap() CommonKeyMap {
	return CommonKeyMap{
		Quit: key.NewBinding(
			key.WithKeys("esc", "ctrl+c"),
			key.WithHelp("esc/ctrl+c", "quit current section."),
		),
	}
}

func (c CommonKeyMap) GetHelpInfo() string {
	s := fmt.Sprintf(HelpInfoPattern, c.Quit.Help().Key, c.Quit.Help().Desc)
	s = HelpStyle.Render(s)
	return s
}
