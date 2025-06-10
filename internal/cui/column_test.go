package cui

import (
	"testing"

	"github.com/charmbracelet/bubbles/table"
)

func TestColumn(t *testing.T) {
	column := NewColumn("search for city:")

	columns := []table.Column{
		{Title: "City", Width: 10},
		{Title: "Country", Width: 10},
		{Title: "Population", Width: 10},
	}

	rows := []table.Row{
		{"Tokyo", "Japan", "37,274,000"},
		{"Delhi", "India", "32,065,760"},
		{"Shanghai", "China", "28,516,904"},
		{"Dhaka", "Bangladesh", "22,478,116"},
		{"São Paulo", "Brazil", "22,429,800"},
		{"Mexico City", "Mexico", "22,085,140"},
		{"Cairo", "Egypt", "21,750,020"},
		{"Beijing", "China", "21,333,332"},
		{"Mumbai", "India", "20,961,472"},
		{"Osaka", "Japan", "19,059,856"},
	}

	column.SetListOptions(
		table.WithColumns(columns),
		table.WithRows(rows),
		table.WithFocused(true),
		table.WithHeight(6),
	)

	column.UpdateViewport()

	if len(column.list.Rows()) == 0 {
		t.Error("expected non-empty list")
	}

	// if _, err := tea.NewProgram(column).Run(); err != nil {
	// 	t.Error(err)
	// }

	// fmt.Println("selected: ", column.Selected())
}
