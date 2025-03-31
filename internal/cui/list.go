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
	rawData        []string
	selected       int
	g              *gocui.Gui
	x0, y0, x1, y1 int
	searcher       *SearcherView
	parent         *gocui.View
}

func NewList(viewName string, g *gocui.Gui) *ListView {
	l := &ListView{
		viewName: viewName,
		g:        g,
	}

	l.searcher = NewSearcher(l)
	return l
}

func (l *ListView) SetCoord(x0, y0, x1, y1 int) {
	l.x0, l.y0, l.x1, l.y1 = x0, y0, x1, y1
}

func (l *ListView) SetData(datalist ...string) {
	l.data = datalist
	l.selected = 0
}

func (l *ListView) SetRawData(datalist ...string) {
	l.rawData = datalist
	l.selected = 0
	if l.data == nil {
		l.data = datalist
	}
}

func (l *ListView) Reset() {
	l.data = l.rawData
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

func (l *ListView) On(g *gocui.Gui, parent *gocui.View) error {
	l.parent = parent
	if v, err := g.SetView(l.viewName, l.x0, l.y0, l.x1, l.y1, 0); err != nil {
		if !gocui.IsUnknownView(err) {
			return err
		}
		v.Title = l.viewName
		v.FgColor = gocui.ColorCyan
		v.Wrap = true
		v.Autoscroll = true
		l.Show(v)
		_, err := SetViewOnTop(l.viewName, g)
		if !gocui.IsUnknownView(err) {
			return err
		}
	}
	return nil
}

func (l *ListView) Off(g *gocui.Gui, _ *gocui.View) error {
	if err := g.DeleteView(l.viewName); err != nil {
		if !gocui.IsUnknownView(err) {
			return err
		}
	}
	_ = g.ForceLayoutAndRedraw()
	if l.parent != nil {
		if _, err := SetViewOnTop(l.parent.Name(), g); err != nil {
			if !gocui.IsUnknownView(err) {
				return err
			}
		}
		l.parent = nil
	}
	return nil
}

func (l *ListView) Search(g *gocui.Gui, v *gocui.View) error {
	if err := l.searcher.On(g, v); err != nil {
		return err
	}
	return nil
}

func (l *ListView) InitKeys() {
	if l.g != nil {
		for _, kb := range l.keybindings {
			_ = kb.Set(l.g, l.viewName)
		}
	}
}
