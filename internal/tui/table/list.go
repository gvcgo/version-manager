package table

import (
	"fmt"
	"sort"
	"strings"

	"github.com/charmbracelet/bubbles/cursor"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/gogf/gf/v2/util/gutil"
	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
)

type ListType string

const (
	SDKList     ListType = "SDKs"
	VersionList ListType = "Versions"
)

type Event func(key string, l *List) tea.Cmd

type KeyEvent struct {
	Event    Event
	HelpInfo string
}

/*
Searchable list.
*/
type List struct {
	Table         Model
	Text          textinput.Model
	WindowHeight  int
	WindowWidth   int
	tableHeader   []Column
	tableRows     []Row
	Type          ListType
	TableKeyEvent map[string]KeyEvent
	NextEvent     string
}

func NewList() (l *List) {
	l = &List{
		Table:         New(),
		Text:          textinput.New(),
		TableKeyEvent: make(map[string]KeyEvent),
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
	l.Table.Blur()
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

func (l *List) SetKeyEventForTable(key string, ke KeyEvent) {
	if key != "" && ke.Event != nil {
		l.TableKeyEvent[key] = ke
	}
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
	if len(r) > 0 {
		return r[0]
	}
	return ""
}

func (l *List) Init() tea.Cmd {
	return textinput.Blink
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
				return l, textinput.Blink
			}
		case "tab":
			if l.Text.Focused() {
				l.Text.Blur()
				l.Table.Focus()
			} else {
				l.Table.Blur()
				l.Text.Focus()
				return l, textinput.Blink
			}
		case "esc", "ctrl+c":
			return l, tea.Quit
		default:
			var cmd tea.Cmd
			if l.Text.Focused() {
				l.Text, cmd = l.Text.Update(msg)
				return l, cmd
			}
			if f, ok := l.TableKeyEvent[keypress]; ok && f.Event != nil {
				cmd = f.Event(keypress, l)
				return l, cmd
			}
			l.Table, cmd = l.Table.Update(msg)
			return l, cmd
		}
	default:
		var cmd tea.Cmd
		if l.Text.Focused() {
			l.Text, cmd = l.Text.Update(msg)
			return l, cmd
		} else {
			l.Table, cmd = l.Table.Update(msg)
			return l, cmd
		}
	}
	return l, nil
}

func (l *List) renderHelpInfo() (count int, s string) {
	lines := []string{gprint.YellowStr("-----------key map-----------")}
	pattern := "→| %-12s  %s"
	if l.Text.Focused() {
		lines = append(lines, fmt.Sprintf(pattern, "enter", "start searching and change focus on table"))
		lines = append(lines, fmt.Sprintf(pattern, "tab", "change focus on table"))
		lines = append(lines, fmt.Sprintf(pattern, "esc", "exit"))
		lines = append(lines, fmt.Sprintf(pattern, "ctrl+c", "exit"))
	} else {
		lines = append(lines, fmt.Sprintf(pattern, "enter", "change focus on search input"))
		lines = append(lines, fmt.Sprintf(pattern, "tab", "change focus on search input"))
		lines = append(lines, fmt.Sprintf(pattern, "esc", "exit"))
		lines = append(lines, fmt.Sprintf(pattern, "ctrl+c", "exit"))
		lines = append(lines, fmt.Sprintf(pattern, "↑/k", "scroll up"))
		lines = append(lines, fmt.Sprintf(pattern, "↓/j", "scroll down"))
		lines = append(lines, fmt.Sprintf(pattern, "g", "goto the first line"))
		lines = append(lines, fmt.Sprintf(pattern, "G", "goto the last line"))

		keyList := []string{}
		for key := range l.TableKeyEvent {
			keyList = append(keyList, key)
		}
		sort.Slice(keyList, func(i, j int) bool {
			return gutil.ComparatorString(keyList[i], keyList[j]) <= 0
		})

		for _, key := range keyList {
			event := l.TableKeyEvent[key]
			lines = append(lines, fmt.Sprintf(pattern, key, event.HelpInfo))
		}
	}
	lines = append(lines, "See docs: https://docs.vmr.us.kg/")
	return len(lines), JoinVertical(lipgloss.Left, lines...)
}

func (l *List) View() string {
	if l.WindowHeight == 0 || l.WindowWidth == 0 {
		return ""
	}
	if l.Text.Focused() {
		l.Text.Placeholder = "Enter something to search for."
		l.Text.Prompt = lipgloss.NewStyle().Copy().Foreground(lipgloss.Color("#32CD32")).Render(">")
	} else {
		l.Text.Placeholder = "Search..."
		l.Text.Prompt = ""
	}

	helpCount, helpInfo := l.renderHelpInfo()
	l.Table.SetHeight(l.WindowHeight - helpCount - 5)
	return lipgloss.JoinVertical(
		lipgloss.Left,
		l.Text.View(),
		"",
		l.Table.View(),
		"",
		lipgloss.NewStyle().Foreground(lipgloss.Color("#D2691E")).Render(helpInfo),
	)
}

func (l *List) Run() {
	p := tea.NewProgram(l, tea.WithAltScreen())
	p.Run()
}
