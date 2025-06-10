package spinner

import (
	"fmt"

	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/gvcgo/version-manager/internal/cui/types"
)

/*
Spinner
*/
var (
	textStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("252")).Render
	spinnerStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("69"))
	helpStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("241")).Render
)

type Spinner struct {
	spinner  spinner.Model
	title    string
	quitting bool
	cancel   types.Hook
}

func NewSpinner(title string) *Spinner {
	s := spinner.New()
	s.Spinner = spinner.Line
	s.Style = spinnerStyle

	return &Spinner{
		spinner:  s,
		title:    title,
		quitting: false,
	}
}

func (s *Spinner) SetCancelHook(hook types.Hook) {
	s.cancel = hook
}

func (s *Spinner) Init() tea.Cmd {
	return s.spinner.Tick
}

func (s *Spinner) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q", "esc":
			return s.quit(nil)
		default:
			return s, nil
		}
	case spinner.TickMsg:
		var cmd tea.Cmd
		s.spinner, cmd = s.spinner.Update(msg)
		if s.quitting {
			return s.quit(cmd)
		}
		return s, cmd
	default:
		if s.quitting {
			return s.quit(nil)
		}
		return s, nil
	}
}

func (s *Spinner) quit(tc tea.Cmd) (tea.Model, tea.Cmd) {
	if s.cancel != nil {
		_ = s.cancel()
	}

	if tc == nil {
		return s, tea.Quit
	} else {
		return s, tea.Batch(tc, tea.Quit)
	}
}

func (s *Spinner) View() (r string) {
	if s.title == "" {
		s.title = "Spinning..."
	}

	r += fmt.Sprintf("\n %s%s%s\n\n", s.spinner.View(), " ", textStyle(s.title))
	r += helpStyle("ctrl+c, esc, q: quit\n")
	return
}

func (s *Spinner) Stop() {
	s.quitting = true
}
