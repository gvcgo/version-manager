package plugin

import (
	"os"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
)

type Plugins struct {
	pls map[string]*Plugin
}

func NewPlugins() *Plugins {
	p := &Plugins{
		pls: make(map[string]*Plugin),
	}
	if ok, _ := gutils.PathIsExist(cnf.GetPluginDir()); !ok {
		p.Update()
	}
	return p
}

func (p *Plugins) Update() {
	if err := UpdatePlugins(); err != nil {
		gprint.PrintError("update plugins failed: %s", err)
	}
}

func (p *Plugins) LoadAll() {
	if len(p.pls) > 0 {
		return
	}

	pDir := cnf.GetPluginDir()
	files, _ := os.ReadDir(pDir)

	for _, f := range files {
		if f.IsDir() || !strings.HasSuffix(f.Name(), ".lua") {
			continue
		}
		pl := NewPlugin(f.Name())
		p.pls[pl.PluginName] = pl
	}
}

func (p *Plugins) GetPlugin(pluginName string) Plugin {
	p.LoadAll()
	if pl, ok := p.pls[pluginName]; ok {
		return *pl
	}
	return Plugin{}
}

func (p *Plugins) GetPluginBySDKName(sdkName string) Plugin {
	p.LoadAll()
	for _, v := range p.pls {
		if v.SDKName == sdkName {
			return *v
		}
	}
	return Plugin{}
}

func (p *Plugins) GetPluginList() (pl []Plugin) {
	p.LoadAll()
	for _, v := range p.pls {
		pl = append(pl, *v)
	}
	return
}

func (p *Plugins) GetPluginSortedRows() (rows []table.Row) {
	p.LoadAll()
	for _, v := range p.pls {
		rows = append(rows, table.Row{
			v.PluginName,
			v.PluginVersion,
			v.SDKName,
			v.Homepage,
		})
	}
	utils.SortVersionAscend(rows)
	return
}
