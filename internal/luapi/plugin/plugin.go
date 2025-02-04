package plugin

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
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
	plugins []Plugin `json:"plugins"`
}

func NewPlugins() *Plugins {
	p := &Plugins{
		plugins: []Plugin{},
	}
	if ok, _ := gutils.PathIsExist(cnf.GetPluginDir()); !ok {
		p.Update()
	}
	return p
}

func (p *Plugins) Update() {
	UpdatePlugins()
}

func (p *Plugins) LoadAll() {
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
		L.DoFile(filepath.Join(pDir, f.Name()))
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
		p.plugins = append(p.plugins, pl)
		ll.Close()
	}
}
