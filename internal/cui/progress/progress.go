package progress

import (
	"fmt"
	"io"
	"os"
	"sync"
	"time"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/progress"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/gvcgo/version-manager/internal/cui/types"
)

const (
	padding        = 2
	maxWidth       = 80
	mbSize   int64 = 1048576
	kbSize   int64 = 1024
)

type (
	ProgressMsg float64
	ErrorMsg    struct{ err error }
	POpt        func(*Progress)
)

func finalPauseCmd() tea.Cmd {
	return tea.Tick(time.Millisecond*750, func(_ time.Time) tea.Msg {
		return nil
	})
}

func WithCancelHook(cancel types.Hook) POpt {
	return func(p *Progress) {
		p.cancel = cancel
	}
}

func WithTotal(total int64) POpt {
	return func(p *Progress) {
		p.total = total
	}
}

func WithProgram(program *tea.Program) POpt {
	return func(p *Progress) {
		p.program = program
	}
}

// Progress for downloadings.
type Progress struct {
	pm        progress.Model
	title     string
	keymap    types.IKeyMap
	total     int64
	completed int64
	lock      *sync.Mutex
	cancel    types.Hook
	err       error
	program   *tea.Program
}

func NewProgress(title string) *Progress {
	pm := progress.New()

	p := &Progress{
		pm:     pm,
		title:  title,
		keymap: types.GetCommonKeyMap(),
		lock:   &sync.Mutex{},
	}
	return p
}

func (p *Progress) SetProgressOptions(opts ...any) {
	for _, opt := range opts {
		if o, ok := opt.(progress.Option); ok {
			o(&p.pm)
		}
		if oo, ok := opt.(POpt); ok {
			oo(p)
		}
	}
}

func (p *Progress) Init() tea.Cmd {
	return nil
}

func (p *Progress) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		km := p.keymap.(types.CommonKeyMap)
		switch {
		case key.Matches(msg, km.Quit):
			if p.cancel != nil {
				if err := p.cancel(); err != nil {
					p.err = err
				}
			}
			return p, tea.Quit
		default:
			return p, nil
		}
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
			cmds = append(cmds, tea.Sequence(finalPauseCmd(), tea.Quit))
		}
		cmds = append(cmds, p.pm.SetPercent(float64(msg)))
		return p, tea.Batch(cmds...)
	// FrameMsg is sent when the progress bar wants to animate itself
	case progress.FrameMsg:
		progressModel, cmd := p.pm.Update(msg)
		p.pm = progressModel.(progress.Model)
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

	extra = lipgloss.JoinHorizontal(0, extra, " ", numbers)
	extra = types.FocusedStyle.Render(extra)
	return extra
}

func (p *Progress) View() string {
	if p.err != nil {
		return "Error downloading: " + p.err.Error() + "\n"
	}

	s := lipgloss.JoinVertical(0, p.getExtraInfo(), p.pm.View())
	return s
}

func (p *Progress) UpdateProgress(toAdd int64) {
	if toAdd <= 0 {
		return
	}
	p.lock.Lock()
	p.completed += toAdd
	if p.program != nil {
		ratio := float64(p.completed) / float64(p.total)
		ratio = min(ratio, 1.0)
		p.program.Send(ProgressMsg(ratio))
	}
	p.lock.Unlock()
}

func (p *Progress) Write(partial []byte) (int, error) {
	length := len(partial)
	p.UpdateProgress(int64(length))
	return length, nil
}

func (p *Progress) Copy(bodyReader io.Reader, storageFile *os.File) (size int64) {
	var err error
	size, err = io.Copy(io.MultiWriter(p, storageFile), bodyReader)
	if err != nil && p.program != nil {
		p.program.Send(ErrorMsg{err})
	}
	return size
}

func (p *Progress) Help() string {
	if p.keymap != nil {
		return p.keymap.GetHelpInfo()
	}
	return ""
}
