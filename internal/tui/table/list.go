package table

import (
	"fmt"

	"github.com/charmbracelet/bubbles/cursor"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

/*
TODO: searchable table
*/
type List struct {
	Table        Model
	Text         textinput.Model
	WindowHeight int
	WindowWidth  int
}

func NewList() (l *List) {
	l = &List{
		Table: New(),
		Text:  textinput.New(),
	}
	l.initTable()
	l.initText()
	return
}

func (l *List) initTable() {
	l.Table.SetColumns([]Column{
		{Title: "SDKName", Width: 20},
	})
	l.Table.SetRows([]Row{
		{"go"},
		{"jdk"},
		{"python"},
	})
	s := DefaultStyles()
	s.Header = s.Header.
		BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(lipgloss.Color("240")).
		BorderBottom(true).
		Bold(false)
	s.Cell = s.Cell.Align(lipgloss.Left)
	s.Selected = s.Selected.
		Foreground(lipgloss.Color("229")).
		Background(lipgloss.Color("57")).
		Bold(false)
	l.Table.SetStyles(s)
}

func (l *List) initText() {
	l.Text.Cursor.SetMode(cursor.CursorBlink)
	l.Text.Prompt = ">"
	l.Text.Placeholder = "Enter something to search for."
	l.Text.CharLimit = -1
	l.Text.Focus()
	l.Text.CursorEnd()
}

func (l *List) Init() tea.Cmd {
	return nil
}

func (l *List) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		l.WindowHeight = msg.Height
		l.WindowWidth = msg.Width
		l.Text.Width = msg.Width - 1
		l.Table.SetWidth(msg.Width - 1)
		l.Table.SetHeight(msg.Height - 5)
	case tea.KeyMsg:
		keypress := msg.String()
		switch keypress {
		case "enter":
			if l.Text.Focused() {
				fmt.Println(l.Text.Value())
			} else if l.Table.Focused() {
				fmt.Println(l.Table.SelectedRow())
			}
		case "tab":
			if l.Text.Focused() {
				l.Text.Blur()
				l.Table.Focus()
			} else {
				l.Table.Blur()
				l.Text.Focus()
			}
		case "esc", "q":
			return l, tea.Quit
		default:
			var cmd tea.Cmd
			if l.Text.Focused() {
				l.Text, cmd = l.Text.Update(msg)
				return l, cmd
			}
			l.Table, cmd = l.Table.Update(msg)
			return l, cmd
		}
	}
	return l, nil
}

func (l *List) View() string {
	if l.WindowHeight == 0 || l.WindowWidth == 0 {
		return ""
	}
	return lipgloss.JoinVertical(
		lipgloss.Left,
		l.Text.View(),
		"",
		l.Table.View(),
	)
}

func (l *List) Run() {
	p := tea.NewProgram(l, tea.WithAltScreen())
	p.Run()
}
