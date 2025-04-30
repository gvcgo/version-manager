package cui

import (
	"fmt"

	"github.com/gvcgo/gocui"
	"github.com/lithammer/fuzzysearch/fuzzy"
)

type SearcherView struct {
	listView       *ListView
	keybindings    []KeyBinding
	x0, y0, x1, y1 int
	parent         *gocui.View
}

func NewSearcher(l *ListView) *SearcherView {
	s := &SearcherView{
		listView: l,
	}

	s.keybindings = append(s.keybindings, NewKeyBinding(
		gocui.KeyEnter,
		gocui.ModNone,
		s.Off,
		"Enter",
		"Submit",
	))
	return s
}

func (s *SearcherView) viewName() string {
	if s.listView != nil {
		return fmt.Sprintf("%s-searcher", s.listView.viewName)
	}
	return "searcher"
}

func (s *SearcherView) SetCoord(x0, y0, x1, y1 int) {
	s.x0, s.y0, s.x1, s.y1 = x0, y0, x1, y1
}

func (s *SearcherView) On(g *gocui.Gui, parent *gocui.View) error {
	s.parent = parent
	if v, err := g.SetView(s.viewName(), s.x0, s.y0, s.x1, s.y1, 0); err != nil {
		if !gocui.IsUnknownView(err) {
			return err
		}
		v.Title = s.viewName()
		v.CanScrollPastBottom = false
		v.Editable = true
		v.FgColor = gocui.ColorCyan
		v.Editor = s
		_, err = SetViewOnTop(s.viewName(), g)
		if !gocui.IsUnknownView(err) {
			return err
		}
	}
	return nil
}

func (s *SearcherView) Off(g *gocui.Gui, _ *gocui.View) error {
	if err := g.DeleteView(s.viewName()); err != nil {
		if !gocui.IsUnknownView(err) {
			return err
		}
	}
	_ = g.ForceLayoutAndRedraw()
	if s.parent != nil {
		if _, err := SetViewOnTop(s.parent.Name(), g); err != nil {
			if !gocui.IsUnknownView(err) {
				return err
			}
		}
		s.parent = nil
	}
	return nil
}

func (s *SearcherView) Edit(v *gocui.View, key gocui.Key, ch rune, mod gocui.Modifier) bool {
	cursorX, cursorY := v.Cursor()
	buffer := v.BufferLines()

	if len(buffer) == 0 {
		buffer = append(buffer, "")
	}
	line := buffer[cursorY]

	switch key {
	case gocui.KeyArrowLeft:
		if cursorX > 0 {
			v.SetCursorX(cursorX - 1)
		}
	case gocui.KeyArrowRight:
		if cursorX < len(line) {
			v.SetCursorX(cursorX + 1)
		}
	case gocui.KeyBackspace, gocui.KeyBackspace2:
		if cursorX > 0 {
			line = line[:cursorX-1] + line[cursorX:]
			v.Clear()
			fmt.Fprint(v, line)
			v.SetCursor(cursorX-1, cursorY)
			_ = s.Search(line)
		}
	case gocui.KeyDelete:
		if cursorX < len(line) {
			line = line[:cursorX] + line[cursorX+1:]
			v.Clear()
			fmt.Fprint(v, line)
			_ = s.Search(line)
		}
	default:
		if ch != 0 {
			line = line[:cursorX] + string(ch) + line[cursorX:]
			v.Clear()
			fmt.Fprint(v, line)
			v.SetCursor(cursorX+1, cursorY)
			_ = s.Search(line)
		}
	}
	return false
}

func (s *SearcherView) Search(line string) error {
	if s.listView != nil {
		if line == "" {
			s.listView.Reset()
		} else {
			data := []string{}
			for _, item := range s.listView.rawData {
				if fuzzy.Match(line, item) {
					data = append(data, item)
				}
			}
			s.listView.SetData(data...)
		}

		if s.listView.g != nil {
			v, err := s.listView.g.View(s.listView.viewName)
			if err != nil && !gocui.IsUnknownView(err) {
				return err
			}
			s.listView.Show(v)
			_ = s.listView.g.ForceLayoutAndRedraw()
		}
	}
	return nil
}

func (s *SearcherView) InitKeys() {
	if s.listView != nil && s.listView.g != nil {
		for _, kb := range s.keybindings {
			_ = kb.Set(s.listView.g, s.viewName())
		}
	}
}
