package cui

import (
	"slices"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/table"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/lithammer/fuzzysearch/fuzzy"
)

type (
	updateListNormallyMsg struct{}
	ColumnKeyMap          struct {
		Quit         key.Binding
		Enter        key.Binding
		LineUp       key.Binding
		LineDown     key.Binding
		PageUp       key.Binding
		PageDown     key.Binding
		HalfPageUp   key.Binding
		HalfPageDown key.Binding
		GotoTop      key.Binding
		GotoBottom   key.Binding
	}
)

var (
	updateListNormallyCmd = func() tea.Msg {
		return updateListNormallyMsg{}
	}
)

func GetColumnKeyMap() ColumnKeyMap {
	return ColumnKeyMap{
		Quit: key.NewBinding(
			key.WithKeys("esc", "ctrl+c"),
			key.WithHelp("esc/ctrl+c", "quit column"),
		),
		Enter: key.NewBinding(
			key.WithKeys("enter"),
			key.WithHelp("enter", "confirm to select current item"),
		),
		LineUp: key.NewBinding(
			key.WithKeys("up", "ctrl+u"),
			key.WithHelp("↑/ctrl+u", "up"),
		),
		LineDown: key.NewBinding(
			key.WithKeys("down", "ctrl+d"),
			key.WithHelp("↓/ctrl+d", "down"),
		),
		PageUp: key.NewBinding(
			key.WithKeys("pgup"),
			key.WithHelp("ctrl+u/pgup", "page up"),
		),
		PageDown: key.NewBinding(
			key.WithKeys("pgdown"),
			key.WithHelp("ctrl+d/pgdn", "page down"),
		),
		GotoTop: key.NewBinding(
			key.WithKeys("home", "ctrl+g"),
			key.WithHelp("ctrl+g/home", "go to start"),
		),
		GotoBottom: key.NewBinding(
			key.WithKeys("end", "ctrl+l"),
			key.WithHelp("ctrl+l/end", "go to end"),
		),
	}
}

func getDefaultTableStyle() table.Styles {
	s := table.DefaultStyles()
	s.Header = s.Header.
		BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(lipgloss.Color("240")).
		BorderBottom(true).
		Bold(false)
	s.Selected = s.Selected.
		Foreground(lipgloss.Color("229")).
		Background(lipgloss.Color("57")).
		Bold(false)
	return s
}

/*
Column for main page.
*/
type Column struct {
	input      textinput.Model
	list       table.Model
	keymap     ColumnKeyMap
	focused    bool
	selected   string
	originRows []table.Row
}

func NewColumn() *Column {
	t := table.New()
	t.SetStyles(getDefaultTableStyle())

	i := textinput.New()
	i.Cursor.Blink = true
	return &Column{
		input:  i,
		list:   t,
		keymap: GetColumnKeyMap(),
	}
}

func (c *Column) Selected() string {
	if c.selected == "" {
		currentRow := c.list.SelectedRow()
		if len(currentRow) > 0 {
			c.selected = currentRow[0]
		}
	}
	return c.selected
}

func (c *Column) fuzzySearch(pattern string) (newRows []table.Row) {
	for _, row := range c.originRows {
		if fuzzy.Match(pattern, row[0]) {
			newRows = append(newRows, slices.Clone(row))
		}
	}
	return
}

func (c *Column) SetListOptions(options ...table.Option) {
	for _, opt := range options {
		if opt != nil {
			opt(&c.list)
		}
	}
	c.originRows = c.list.Rows()
}

func (c *Column) Init() tea.Cmd {
	cmd := c.input.Focus()
	return tea.Batch(cmd, textinput.Blink)
}

func (c *Column) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch {
		case key.Matches(msg, c.keymap.Quit):
			return c, tea.Quit
		case key.Matches(msg, c.keymap.Enter):
			currentRow := c.list.SelectedRow()
			if len(currentRow) > 0 {
				c.selected = currentRow[0]
			}
			return c, updateListNormallyCmd
		case key.Matches(msg, c.keymap.PageUp):
			c.list.MoveUp(c.list.Height())
			return c, updateListNormallyCmd
		case key.Matches(msg, c.keymap.PageDown):
			c.list.MoveDown(c.list.Height())
			return c, updateListNormallyCmd
		case key.Matches(msg, c.keymap.LineUp):
			c.list.MoveUp(1)
			return c, updateListNormallyCmd
		case key.Matches(msg, c.keymap.LineDown):
			c.list.MoveDown(1)
			return c, updateListNormallyCmd
		case key.Matches(msg, c.keymap.GotoTop):
			c.list.GotoTop()
			return c, updateListNormallyCmd
		case key.Matches(msg, c.keymap.GotoBottom):
			c.list.GotoBottom()
			return c, updateListNormallyCmd
		default:
			// TODO: cmd
			c.input, _ = c.input.Update(msg)

			pattern := c.input.Value()
			// fmt.Println("---pattern: ", pattern)
			newRows := c.fuzzySearch(pattern)
			c.list.SetRows(newRows)
			return c, updateListNormallyCmd
		}
	case updateListNormallyMsg:
		c.UpdateViewport()
		return c, nil
	default:
		return c, nil
	}
}

func (c *Column) View() string {
	var (
		inputView = c.input.View()
		listView  = c.list.View()
	)
	if c.input.Focused() {
		inputView = focusedStyle.Render(inputView)
	} else {
		inputView = bluredStyle.Render(inputView)
	}

	if c.list.Focused() {
		listView = focusedStyle.Render(listView)
	} else {
		listView = bluredStyle.Render(listView)
	}

	s := lipgloss.JoinVertical(
		0,
		inputView,
		listView,
	)
	return s
}

func (c *Column) Focus() tea.Cmd {
	c.focused = true
	return c.input.Focus()
}

func (c *Column) Blur() {
	c.input.Blur()
	c.list.Blur()
	c.focused = false
}

func (c *Column) UpdateViewport() {
	c.list.UpdateViewport()
}
