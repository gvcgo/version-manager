package cui

import "github.com/charmbracelet/lipgloss"

var (
	focusedStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#00AF00"))
	bluredStyle  = lipgloss.NewStyle().Foreground(lipgloss.Color("#B0B0B0"))
)

type Hook func() error
