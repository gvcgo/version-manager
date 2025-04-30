package vui

import (
	"fmt"
	"sync"
	"time"

	"github.com/charmbracelet/bubbles/progress"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

func finalPause() tea.Cmd {
	return tea.Tick(time.Millisecond*750, func(_ time.Time) tea.Msg {
		return nil
	})
}

const (
	padding  = 2
	maxWidth = 80
	mbSize   = 1048576
	kbSize   = 1024
)

var (
	helpStyle  = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFA500")).Render
	titleStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("#abebc6")).Render
)

type (
	ProgressMsg float64
	ErrorMsg    string
)

type Progress struct {
	model      progress.Model
	total      int64
	downloaded int64
	lock       *sync.Mutex
	sweep      func()
	title      string
	errMsg     ErrorMsg
}

func (p *Progress) SetTitle(title string) {
	p.title = title
}

func (p *Progress) Init() tea.Cmd {
	return nil
}

func (p *Progress) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		if msg.String() == "q" {
			if p.sweep != nil {
				p.sweep()
			}
			return p, tea.Quit
		}
		return p, nil
	case tea.WindowSizeMsg:
		p.model.Width = min(msg.Width-padding*2-4, maxWidth)
		return p, nil
	case ProgressMsg:
		var cmds []tea.Cmd
		if msg >= 1.0 {
			cmds = append(cmds, tea.Sequence(finalPause(), tea.Quit))
		}
		cmds = append(cmds, p.model.SetPercent(float64(msg)))
		return p, tea.Batch(cmds...)

	case ErrorMsg:
		p.errMsg = ErrorMsg(msg)
		return p, tea.Quit
	// FrameMsg is sent when the progress bar wants to animate itself
	case progress.FrameMsg:
		progressModel, cmd := p.model.Update(msg)
		p.model = progressModel.(progress.Model)
		return p, cmd
	default:
		return p, nil
	}
}

func (p *Progress) getDownloadInfo() (downloaded string) {
	if p.total > int64(mbSize) {
		downloaded = fmt.Sprintf(
			"%.2f/%.2f MB",
			float64(p.downloaded)/float64(mbSize),
			float64(p.total)/float64(mbSize),
		)
	} else {
		downloaded = fmt.Sprintf(
			"%.2f/%.2f KB",
			float64(p.downloaded)/float64(kbSize),
			float64(p.total)/float64(kbSize),
		)
	}
	return
}

func (p *Progress) getDownloadedPercentage() (percentage string) {
	percentage = fmt.Sprintf("%.2f%%", float64(p.downloaded)/float64(p.total)*100)
	return
}

func (p *Progress) View() string {
	if p.errMsg != "" {
		return fmt.Sprintf("[%s] Errored: %v\n", p.title, p.errMsg)
	}
	title := fmt.Sprintf("[%s] %s %s", p.title, p.getDownloadInfo(), p.getDownloadedPercentage())

	return lipgloss.JoinVertical(
		lipgloss.Left, titleStyle(title),
		p.model.View(),
		helpStyle(`Press "q" to quit.`),
	)
}

func (p *Progress) Write(partial []byte) (int, error) {
	p.lock.Lock()
	p.downloaded += int64(len(partial))
	if p.total > 0 {
		ratio := float64(p.downloaded) / float64(p.total)
		p.Update(ProgressMsg(ratio))
	}
	p.lock.Unlock()
	return len(partial), nil
}
