package cui

import (
	"log"

	"github.com/gvcgo/gocui"
)

/*
TODO: console UI
*/

type VmrTUI struct {
	g *gocui.Gui
}

func New() (vt *VmrTUI) {
	opt := gocui.NewGuiOpts{
		OutputMode: gocui.OutputNormal,
	}
	g, err := gocui.NewGui(opt)
	g.Cursor = false
	g.Mouse = true
	g.InputEsc = true
	g.SupportOverlaps = true
	if err != nil {
		log.Panicln(err)
		return nil
	}
	vt = &VmrTUI{
		g: g,
	}
	return
}

func (vt *VmrTUI) Close() {
	if vt.g != nil {
		vt.g.Close()
	}
}
