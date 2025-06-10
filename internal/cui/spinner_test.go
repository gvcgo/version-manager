package cui

import (
	"testing"
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

func TestSpinner(t *testing.T) {
	s := NewSpinner("test spinner")

	// go func() {
	// 	fmt.Println("stop spinning")
	// 	s.Stop()
	// }()

	go func() {
		if _, err := tea.NewProgram(s).Run(); err != nil {
			t.Error(err)
		}
	}()
	time.Sleep(time.Second)
}
