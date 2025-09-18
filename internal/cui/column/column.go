package column

import (
	"fmt"
	"slices"

	"github.com/charmbracelet/bubbles/cursor"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/table"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/gvcgo/version-manager/internal/cui/types"
	"github.com/lithammer/fuzzysearch/fuzzy"
)

type (
	updateListNormallyMsg struct{}
	blinkMsg              struct{ msg any }
	ColumnKeyMap          struct {
		Quit       key.Binding
		Enter      key.Binding
		LineUp     key.Binding
		LineDown   key.Binding
		PageUp     key.Binding
		PageDown   key.Binding
		GotoTop    key.Binding
		GotoBottom key.Binding
	}
)

func (c ColumnKeyMap) GetHelpInfo() string {
	s := lipgloss.JoinVertical(
		0,
		fmt.Sprintf(types.HelpInfoPattern, c.Quit.Help().Key, c.Quit.Help().Desc),
		fmt.Sprintf(types.HelpInfoPattern, c.Enter.Help().Key, c.Enter.Help().Desc),
		fmt.Sprintf(types.HelpInfoPattern, c.LineUp.Help().Key, c.LineUp.Help().Desc),
		fmt.Sprintf(types.HelpInfoPattern, c.LineDown.Help().Key, c.LineDown.Help().Desc),
		fmt.Sprintf(types.HelpInfoPattern, c.PageUp.Help().Key, c.PageUp.Help().Desc),
		fmt.Sprintf(types.HelpInfoPattern, c.PageDown.Help().Key, c.PageDown.Help().Desc),
		fmt.Sprintf(types.HelpInfoPattern, c.GotoTop.Help().Key, c.GotoTop.Help().Desc),
		fmt.Sprintf(types.HelpInfoPattern, c.GotoBottom.Help().Key, c.GotoBottom.Help().Desc),
	)
	s = types.HelpStyle.Render(s)
	return s
}

var (
	updateListNormallyCmd = func() tea.Msg {
		return updateListNormallyMsg{}
	}
	blinkCmd = func() tea.Msg {
		return blinkMsg{msg: textinput.Blink()}
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
	keymap     types.IKeyMap
	focused    bool
	selected   string
	originRows []table.Row
	title      string
}

func NewColumn(title string) *Column {
	t := table.New()
	t.SetStyles(getDefaultTableStyle())

	i := textinput.New()
	i.Cursor.Blink = true
	i.Cursor.SetMode(cursor.CursorBlink)
	return &Column{
		input:  i,
		list:   t,
		keymap: GetColumnKeyMap(),
		title:  title,
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

func (c *Column) SetTitle(title string) {
	c.title = title
}

func (c *Column) Init() tea.Cmd {
	cmd := c.input.Focus()
	return tea.Batch(cmd, blinkCmd)
}

func (c *Column) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		km := c.keymap.(ColumnKeyMap)
		switch {
		case key.Matches(msg, km.Quit):
			return c, tea.Quit
		case key.Matches(msg, km.Enter):
			currentRow := c.list.SelectedRow()
			if len(currentRow) > 0 {
				c.selected = currentRow[0]
			}
			return c, updateListNormallyCmd
		case key.Matches(msg, km.PageUp):
			c.list.MoveUp(c.list.Height())
			return c, updateListNormallyCmd
		case key.Matches(msg, km.PageDown):
			c.list.MoveDown(c.list.Height())
			return c, updateListNormallyCmd
		case key.Matches(msg, km.LineUp):
			c.list.MoveUp(1)
			return c, updateListNormallyCmd
		case key.Matches(msg, km.LineDown):
			c.list.MoveDown(1)
			return c, updateListNormallyCmd
		case key.Matches(msg, km.GotoTop):
			c.list.GotoTop()
			return c, updateListNormallyCmd
		case key.Matches(msg, km.GotoBottom):
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
	case blinkMsg:
		var cmd tea.Cmd
		c.input.Cursor, cmd = c.input.Cursor.Update(msg.msg)
		return c, cmd
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
		inputView = types.FocusedStyle.Render(inputView)
	} else {
		inputView = types.BluredStyle.Render(inputView)
	}

	if c.list.Focused() {
		listView = types.FocusedStyle.Render(listView)
	} else {
		listView = types.BluredStyle.Render(listView)
	}

	s := lipgloss.JoinVertical(
		0,
		inputView,
		listView,
	)
	if c.title != "" {
		title := fmt.Sprintf("%s %s", "●", c.title)
		if c.Focused() {
			title = types.FocusedStyle.Render(title)
		} else {
			title = types.BluredStyle.Render(title)
		}
		s = lipgloss.JoinVertical(0, title, s)
	}
	return s
}

func (c *Column) Focus() tea.Cmd {
	c.focused = true
	return c.input.Focus()
}

func (c *Column) Focused() bool {
	return c.input.Focused() || c.list.Focused()
}

func (c *Column) Blur() {
	c.input.Blur()
	c.list.Blur()
	c.focused = false
}

func (c *Column) Help() string {
	if c.keymap != nil {
		return c.keymap.GetHelpInfo()
	}
	return ""
}

func (c *Column) UpdateViewport() {
	c.list.UpdateViewport()
}
