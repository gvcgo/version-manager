package spinner

import (
	"fmt"
	"testing"

	_ "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/version-manager/internal/cui/types"
)

func TestSpinner(t *testing.T) {
	s := NewSpinner("test spinner")

	var h types.Hook = func() error {
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
