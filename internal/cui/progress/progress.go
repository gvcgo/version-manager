package progress

import (
	"sync"
	"time"

	"github.com/charmbracelet/bubbles/progress"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/version-manager/internal/cui/types"
)

type tickMsg time.Time

func tickCmd() tea.Cmd {
	return tea.Tick(time.Second*1, func(t time.Time) tea.Msg {
		return tickMsg(t)
	})
}

// Progress for downloadings.
type Progress struct {
	pm        progress.Model
	title     string
	total     int
	completed int
	lock      *sync.Mutex
	cancel    types.Hook
}
