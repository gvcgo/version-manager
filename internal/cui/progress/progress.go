package progress

import (
	"fmt"
	"io"
	"os"
	"sync"
	"time"

	"github.com/charmbracelet/bubbles/progress"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/gvcgo/version-manager/internal/cui/types"
)

type tickMsg time.Time

func tickCmd() tea.Cmd {
	return tea.Tick(time.Second*1, func(t time.Time) tea.Msg {
		return tickMsg(t)
	})
}

const (
	padding        = 2
	maxWidth       = 80
	mbSize   int64 = 1048576
	kbSize   int64 = 1024
)

type ProgressMsg float64

type ErrorMsg struct{ err error }

func finalPause() tea.Cmd {
	return tea.Tick(time.Millisecond*750, func(_ time.Time) tea.Msg {
		return nil
	})
}

// Progress for downloadings.
type Progress struct {
	pm        progress.Model
	title     string
	total     int64
	completed int64
	lock      *sync.Mutex
	cancel    types.Hook
	err       error
}

func NewProgress(title string) *Progress {
	pm := progress.New()

	p := &Progress{
		pm:    pm,
		title: title,
		lock:  &sync.Mutex{},
	}
	return p
}

func (p *Progress) SetCancelHook(cancel types.Hook) {
	p.cancel = cancel
}

func (p *Progress) SetProgressOptions(options ...progress.Option) {
	for _, opt := range options {
		opt(&p.pm)
	}
}

func (p *Progress) Init() tea.Cmd {
	return tickCmd()
}

func (p *Progress) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		if msg.String() == "q" {
			if p.cancel != nil {
				p.cancel()
			}
			return p, tea.Quit
		}
		return p, nil
	case tea.WindowSizeMsg:
		p.pm.Width = msg.Width - padding*2 - 4
		p.pm.Width = min(p.pm.Width, maxWidth)
		return p, nil
	case ErrorMsg:
		p.err = msg.err
		return p, tea.Quit
	case ProgressMsg:
		var cmds []tea.Cmd
		if msg >= 1.0 {
			cmds = append(cmds, tea.Sequence(finalPause(), tea.Quit))
		} else if p.err != nil {
			if p.cancel != nil {
				p.cancel()
			}
			cmds = append(cmds, tea.Quit)
		}
		cmds = append(cmds, p.pm.SetPercent(float64(msg)))
		return p, tea.Batch(cmds...)
	// FrameMsg is sent when the progress bar wants to animate itself
	case progress.FrameMsg:
		pm, cmd := p.pm.Update(msg)
		p.pm, _ = pm.(progress.Model)
		if p.err != nil {
			if p.cancel != nil {
				p.cancel()
			}
			cmd = tea.Batch(cmd, tea.Quit)
		}
		return p, cmd
	default:
		return p, nil
	}
}

func (p *Progress) getExtraInfo() string {
	extra := p.title

	var numbers string
	if p.total > int64(mbSize) {
		numbers = fmt.Sprintf(
			"[%.2f/%.2f MB]",
			float64(p.completed)/float64(mbSize),
			float64(p.total)/float64(mbSize),
		)
	} else {
		numbers = fmt.Sprintf(
			"[%.2f/%.2f KB]",
			float64(p.completed)/float64(kbSize),
			float64(p.total)/float64(kbSize),
		)
	}

	extra = lipgloss.JoinHorizontal(0.5, extra, " ", numbers)
	extra = types.FocusedStyle.Render(extra)
	return extra
}

func (p *Progress) View() string {
	if p.err != nil {
		return "Error downloading: " + p.err.Error() + "\n"
	}

	s := lipgloss.JoinVertical(0.5, p.getExtraInfo(), p.pm.View())
	return s
}

func (p *Progress) UpdateProgress(toAdd int64) {
	if toAdd <= 0 {
		return
	}
	p.lock.Lock()
	p.completed += toAdd
	if p.total > 0 {
		ratio := float64(p.completed) / float64(p.total)
		_ = p.pm.SetPercent(ratio)
	}
	p.lock.Unlock()
}

func (p *Progress) Write(partial []byte) (int, error) {
	length := len(partial)
	p.UpdateProgress(int64(length))
	return length, nil
}

func (p *Progress) Copy(bodyReader io.Reader, storageFile *os.File) (size int64) {
	size, p.err = io.Copy(io.MultiWriter(p, storageFile), bodyReader)
	return size
}
