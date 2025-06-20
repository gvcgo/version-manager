package plugin

import (
	"testing"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/utils"
)

func TestPlugins(t *testing.T) {
	pluginsDir := cnf.GetPluginDir()
	if !utils.PathIsExist(pluginsDir) {
		return
	}
	ps := NewPlugins()
	p := ps.GetPlugin("go")

	if p.PluginName != "go" {
		t.Errorf("Expected plugin name 'go', got '%s'", p.PluginName)
	}
}
