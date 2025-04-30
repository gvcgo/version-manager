package cui

import (
	"fmt"

	"github.com/gvcgo/gocui"
)

type KeyHandler func(g *gocui.Gui, v *gocui.View) error

type KeyBind struct {
	key      any
	keyStr   string
	keyDescr string
	modifier gocui.Modifier
	handler  KeyHandler
}

func NewKeyBinding(key any,
	modifier gocui.Modifier,
	handler KeyHandler,
	keyStr string,
	keyDecr string) (k *KeyBind) {
	return &KeyBind{
		key:      key,
		modifier: modifier,
		handler:  handler,
		keyStr:   keyDecr,
		keyDescr: keyDecr,
	}
}

func (k *KeyBind) Set(g *gocui.Gui, viewName string) error {
	return g.SetKeybinding(viewName, k.key, k.modifier, k.handler)
}

func (k *KeyBind) Unset(g *gocui.Gui, viewName string) error {
	return g.DeleteKeybinding(viewName, k.key, k.modifier)
}

func (k *KeyBind) HelpInfo() string {
	return fmt.Sprintf("<%s>%s", k.keyStr, k.keyDescr)
}

type KeyBinding interface {
	Set(*gocui.Gui, string) error
	Unset(*gocui.Gui, string) error
	HelpInfo() string
}
