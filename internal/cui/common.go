package cui

import "github.com/gvcgo/gocui"

func SetViewOnTop(viewName string, g *gocui.Gui) (*gocui.View, error) {
	if _, err := g.SetCurrentView(viewName); err != nil {
		return nil, err
	}
	v, err := g.SetViewOnTop(viewName)
	_ = g.ForceLayoutAndRedraw()
	return v, err
}
