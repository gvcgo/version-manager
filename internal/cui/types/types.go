package types

import (
	"github.com/charmbracelet/lipgloss"
)

var (
	FocusedStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#00AF00"))
	BluredStyle  = lipgloss.NewStyle().Foreground(lipgloss.Color("#B0B0B0"))
)

type Hook func() error
