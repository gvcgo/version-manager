package progress

import (
	"fmt"
	"testing"
)

func TestProgress(t *testing.T) {
	title := "mocking-download.zip"
	p := NewProgress(title)
	p.SetTotal(100000)
	p.SetCancelHook(func() error {
		fmt.Println("in hook")
		return nil
	})

	if p.title != title {
		t.Errorf("expected title to be %s, got %s", title, p.title)
	}

	// pro := tea.NewProgram(p)
	// if pro == nil {
	// 	t.Error("cannot init program!")
	// }

	// p.SetProgram(pro)
	// go func() {
	// 	for _ = range 100 {
	// 		p.UpdateProgress(1000)
	// 		time.Sleep(time.Millisecond * 100)
	// 	}
	// }()

	// if _, err := pro.Run(); err != nil {
	// 	t.Error(err)
	// }

	// fmt.Println("completed: ", p.completed)
	// time.Sleep(time.Second * 2)
}
