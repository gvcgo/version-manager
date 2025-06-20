package cinstaller

import (
	"fmt"
	"testing"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/plugin"
	"github.com/gvcgo/version-manager/internal/utils"
)

func TestInstallerVersionList(t *testing.T) {
	pluginsDir := cnf.GetPluginDir()
	if !utils.PathIsExist(pluginsDir) {
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
