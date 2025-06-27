package help

import (
	"fmt"

	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/gvcgo/version-manager/internal/cui/types"
)

/*
Shows help info.
*/
type Help struct {
	title   string
	content string
	vp      viewport.Model
	ready   bool
}

func NewHelp(title string) *Help {
	return &Help{
		title: title,
		vp:    viewport.New(0, 0),
	}
}

func (h *Help) SetContent(content string) {
	h.content = content
	h.vp.SetContent(content)
}

func (h *Help) Init() tea.Cmd {
	return nil
}

func (h *Help) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	var (
		cmd  tea.Cmd
		cmds []tea.Cmd
	)

	switch msg := msg.(type) {
	case tea.KeyMsg:
		if k := msg.String(); k == "ctrl+c" || k == "q" || k == "esc" {
			return h, tea.Quit
		}
	case tea.WindowSizeMsg:
		if !h.ready {
			// Since this program is using the full size of the viewport we
			// need to wait until we've received the window dimensions before
			// we can initialize the viewport. The initial dimensions come in
			// quickly, though asynchronously, which is why we wait for them
			// here.
			h.vp = viewport.New(msg.Width, msg.Height-2)
			h.vp.YPosition = 1
			h.vp.SetContent(h.content)
			h.ready = true
		} else {
			h.vp.Width = msg.Width
			h.vp.Height = msg.Height - 2
		}
	}

	// Handle keyboard and mouse events in the viewport
	h.vp, cmd = h.vp.Update(msg)
	cmds = append(cmds, cmd)

	return h, tea.Batch(cmds...)
}

func (h *Help) View() string {
	if !h.ready {
		return "\n  Initializing..."
	}
	header := types.FocusedStyle.Render(fmt.Sprintf("shortcuts for %s", h.title))
	footer := types.FocusedStyle.Render("press <q>,<esc>,<ctrl+c> to hide help info")
	return lipgloss.JoinVertical(0, header, h.content, footer)
}
