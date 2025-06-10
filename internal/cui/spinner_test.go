package cui

import (
	"fmt"
	"testing"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

func TestSpinner(t *testing.T) {
	s := NewSpinner("test spinner")

	go func() {
		time.Sleep(time.Second)
		fmt.Println("stop spinning")
		s.Stop()
	}()

	if _, err := tea.NewProgram(s).Run(); err != nil {
		t.Error(err)
	}
}
