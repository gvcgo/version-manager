package cui

import (
	"fmt"

	"github.com/gvcgo/gocui"
)

type ListView struct {
	viewName       string
	keybindings    []KeyBinding
	show           bool
	data           []string
	selected       int
	g              *gocui.Gui
	x0, y0, x1, y1 int
}

func NewList(viewName string, g *gocui.Gui) *ListView {
	return &ListView{
		viewName: viewName,
		g:        g,
	}
}

func (l *ListView) SetCoord(x0, y0, x1, y1 int) {
	l.x0, l.y0, l.x1, l.y1 = x0, y0, x1, y1
}

func (l *ListView) SetData(datalist ...string) {
	l.data = datalist
	l.selected = 0
}

func (l *ListView) Show(v *gocui.View) {
	if l.g == nil {
		return
	}
	v.Clear()

	dataLen := len(l.data)
	_, vHeight := v.Size()
	vHeight -= 2

	dataList := l.data
	selected := dataList[l.selected]
	if dataLen > vHeight {
		if l.selected < dataLen-vHeight {
			dataList = l.data[l.selected : l.selected+vHeight]
		} else {
			dataList = l.data[dataLen-vHeight:]
		}
	}

	for _, line := range dataList {
		if selected == line {
			fmt.Fprintf(v, "> %s\n", line)
		} else {
			fmt.Fprintf(v, "  %s\n", line)
		}
	}
}

func (l *ListView) Down(g *gocui.Gui, v *gocui.View) error {
	if l.show && l.g != nil {
		l.selected = (l.selected + 1) % len(l.data)
		g.Update(func(g *gocui.Gui) error {
			l.Show(v)
			return nil
		})
	}
	return nil
}

func (l *ListView) Up(g *gocui.Gui, v *gocui.View) error {
	if l.show {
		l.selected = (l.selected - 1 + len(l.data)) % len(l.data)
		g.Update(func(g *gocui.Gui) error {
			l.Show(v)
			return nil
		})
	}
	return nil
}

func (l *ListView) Search(g *gocui.Gui, v *gocui.View) error {
	return nil
}
