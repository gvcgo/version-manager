package vui

import (
	"os"

	"github.com/charmbracelet/lipgloss"
	"golang.org/x/term"
)

var (
	UnfocusedBorderStyle = lipgloss.NewStyle().
				Border(lipgloss.RoundedBorder()).
				BorderForeground(lipgloss.Color("#f2f3f4")).
				Padding(1).
				Margin(1)

	FocusedBorderStyle = lipgloss.NewStyle().
				Border(lipgloss.RoundedBorder()).
				BorderForeground(lipgloss.Color("#2ecc71")).
				Padding(1).
				Margin(1)

	helpStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFA500")).Render
)

func GetTermSize() (height, width int, err error) {
	fd := os.Stdout.Fd()
	return term.GetSize(int(fd))
}
