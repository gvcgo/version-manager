package table

import (
	"strings"

	"github.com/charmbracelet/bubbles/cursor"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type ListType string

const (
	SDKList     ListType = "SDKs"
	VersionList ListType = "Versions"
)

/*
TODO: searchable table
*/
type List struct {
	Table        Model
	Text         textinput.Model
	WindowHeight int
	WindowWidth  int
	tableHeader  []Column
	tableRows    []Row
	Type         ListType
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

func (l *List) SetListType(t ListType) {
	l.Type = t
}

func (l *List) SetHeader(header []Column) {
	l.tableHeader = header
	l.Table.SetColumns(header)
}

func (l *List) SetRows(rows []Row) {
	l.tableRows = rows
	l.Table.SetRows(rows)
}

func (l *List) Search() {
	s := l.Text.Value()
	newRows := []Row{}
	for _, row := range l.tableRows {
		if strings.HasPrefix(row[0], s) {
			newRows = append(newRows, row)
		}
	}
	l.Table.SetRows(newRows)
}

func (l *List) GetSelected() string {
	r := l.Table.SelectedRow()
	return r[0]
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
				l.Search()
				l.Text.Blur()
				l.Table.Focus()
				l.Table.SetCursor(0)
			} else if l.Table.Focused() {
				l.Table.Blur()
				l.Text.Focus()
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
	if l.Text.Focused() {
		l.Text.Prompt = lipgloss.NewStyle().Copy().Foreground(lipgloss.Color("#32CD32")).Render(">")
	} else {
		l.Text.Prompt = ""
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
