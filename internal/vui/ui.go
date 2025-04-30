package vui

import (
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/table"
	"github.com/charmbracelet/bubbles/textinput"
	"github.com/charmbracelet/bubbles/viewport"
	"github.com/charmbracelet/lipgloss"
)

type VUI struct {
	sdkInput   textinput.Model
	sdkList    table.Model
	sdkRows    []table.Row
	verInput   textinput.Model
	verList    table.Model
	verRows    []table.Row
	renderMain bool
	progress   Progress
	spinner    spinner.Model
	panel      viewport.Model // help info
}

func NewVUI() *VUI {
	vui := &VUI{
		renderMain: true,
	}
	vui.InitiateUI()
	return vui
}

func (v *VUI) InitiateUI() {
	h, w, err := GetTermSize()
	if err != nil {
		panic(err)
	}
	// left
	v.sdkInput = textinput.New()
	v.sdkInput.Placeholder = "search by sdk name"
	v.sdkInput.Focus()
	v.sdkInput.Width = w/3 - 2
	v.sdkInput.CharLimit = 30

	v.sdkList = table.New()
	v.sdkList.SetWidth(w/3 - 2)
	v.sdkList.SetHeight(h - 5)
	ss := table.DefaultStyles()
	ss.Header = ss.Header.BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(lipgloss.Color("240")).
		BorderBottom(true).
		Bold(false)
	ss.Selected = ss.Selected.
		Foreground(lipgloss.Color("229")).
		Background(lipgloss.Color("57")).
		Bold(false)
	v.sdkList.SetStyles(ss)

	// right
	v.verInput = textinput.New()
	v.verInput.Placeholder = "search by version name"
	v.verInput.Focus()
	v.verInput.Width = w*2/3 - 2
	v.verInput.CharLimit = 30

	v.verList = table.New()
	v.verList.SetWidth(w*2/3 - 2)
	v.verList.SetHeight(h - 5)
	vs := table.DefaultStyles()
	vs.Header = vs.Header.BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(lipgloss.Color("240")).
		BorderBottom(true).
		Bold(false)
	vs.Selected = ss.Selected.
		Foreground(lipgloss.Color("229")).
		Background(lipgloss.Color("57")).
		Bold(false)
	v.verList.SetStyles(vs)

	// progress
}
