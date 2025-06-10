package cui

import (
	"fmt"
	"testing"

	_ "github.com/charmbracelet/bubbletea"
)

func TestSpinner(t *testing.T) {
	s := NewSpinner("test spinner")

	var h Hook = func() error {
		fmt.Println("in hook")
		return nil
	}
	s.SetCancelHook(h)

	if s.cancel != nil {
		err := s.cancel()
		if err != nil {
			t.Error(err)
		}
	}
	// TODO: github test do not support tea program

	// go func() {
	// 	fmt.Println("stop spinning")
	// 	s.Stop()
	// }()

	// 	if _, err := tea.NewProgram(s).Run(); err != nil {
	// 		t.Error(err)
	// 	}
	// time.Sleep(time.Second)
}
