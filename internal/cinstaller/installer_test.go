package cinstaller

import (
	"fmt"
	"path/filepath"
	"testing"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/gvcgo/version-manager/internal/utils"
)

func TestInstallerVersionList(t *testing.T) {
	pluginPath := filepath.Join(cnf.GetPluginDir(), "go.lua")
	if !utils.PathIsExist(pluginPath) {
		return
	}
	plugins := plugin.NewPlugins()
	p := plugins.GetPlugin("go")
	i := New(&p)
	vl, err := i.GetVersionList()
	if err != nil {
		t.Errorf("Error getting version list: %v", err)
	} else {
		sortedList := vl.SortDesc()
		fmt.Println(sortedList)
	}
}
