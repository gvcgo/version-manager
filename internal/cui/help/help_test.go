package help

import (
	"testing"

	"github.com/gvcgo/version-manager/internal/cui/column"
)

func TestHelp(t *testing.T) {
	c := column.NewColumn("test column")
	h := NewHelp("test help")
	h.SetContent(c.Help())

	if h.content == "" {
		t.Error("content is empty")
	}

	// p := tea.NewProgram(h)
	// if p == nil {
	// 	t.Error("cannot init program!")
	// }

	// if _, err := p.Run(); err != nil {
	// 	t.Error(err)
	// }
}
