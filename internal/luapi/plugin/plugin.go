package plugin

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
)

type Plugin struct {
	FileName      string `json:"file_name"`
	PluginName    string `json:"plugin_name"`
	PluginVersion string `json:"plugin_version"`
	SDKName       string `json:"sdk_name"`
	Prequisite    string `json:"prequisite"`
	Homepage      string `json:"homepage"`
}

type Plugins struct {
	pls map[string]Plugin
}

func NewPlugins() *Plugins {
	p := &Plugins{
		pls: make(map[string]Plugin),
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
		pl := Plugin{
			FileName: f.Name(),
		}
		ll := lua_global.NewLua()
		L := ll.L
		if err := L.DoFile(filepath.Join(pDir, f.Name())); err != nil {
			continue
		}
		pl.PluginName = GetConfItemFromLua(L, PluginName)
		if pl.PluginName == "" {
			continue
		}
		pl.PluginVersion = GetConfItemFromLua(L, PluginVersion)
		pl.SDKName = GetConfItemFromLua(L, SDKName)
		if pl.SDKName == "" {
			continue
		}
		pl.Prequisite = GetConfItemFromLua(L, Prequisite)
		pl.Homepage = GetConfItemFromLua(L, Homepage)
		if pl.Homepage == "" {
			continue
		}
		if !DoLuaItemExist(L, InstallerConfig) || !DoLuaItemExist(L, Crawler) {
			continue
		}
		p.pls[pl.PluginName] = pl
		ll.Close()
	}
}

func (p *Plugins) GetPlugin(pluginName string) Plugin {
	p.LoadAll()
	if pl, ok := p.pls[pluginName]; ok {
		return pl
	}
	return Plugin{}
}

func (p *Plugins) GetPluginBySDKName(sdkName string) Plugin {
	p.LoadAll()
	for _, v := range p.pls {
		if v.SDKName == sdkName {
			return v
		}
	}
	return Plugin{}
}

func (p *Plugins) GetPluginList() (pl []Plugin) {
	p.LoadAll()
	for _, v := range p.pls {
		pl = append(pl, v)
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
