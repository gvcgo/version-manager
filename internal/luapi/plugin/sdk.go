package plugin

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
	lua "github.com/yuin/gopher-lua"
)

/*
SDK Versions
*/
type Versions struct {
	Vs             lua_global.VersionList
	PluginFileName string
	Lua            *lua_global.Lua
}

func NewVersions(fileName string) (v *Versions) {
	return &Versions{
		PluginFileName: fileName,
	}
}

func (v *Versions) loadPlugin() error {
	if v.Lua == nil {
		v.Lua = lua_global.NewLua()
	}
	pDir := cnf.GetPluginDir()
	fPath := filepath.Join(pDir, v.PluginFileName)
	if ok, _ := gutils.PathIsExist(fPath); !ok {
		return fmt.Errorf("plugin file not found: %s", v.PluginFileName)
	}
	if err := v.Lua.L.DoFile(fPath); err != nil {
		return fmt.Errorf("failed to load plugin file: %s", err)
	}
	return nil
}

func (v *Versions) loadFromCache(pluginName string) {
	cacheFilePath := filepath.Join(cnf.GetCacheDir(), pluginName)
	if ok, _ := gutils.PathIsExist(cacheFilePath); !ok {
		return
	}
	lastModifiedTime := utils.GetFileLastModifiedTime(cacheFilePath)
	if lastModifiedTime >= 86400 {
		return
	}
	if content, err := os.ReadFile(cacheFilePath); err == nil {
		err = json.Unmarshal(content, &v.Vs)
		if err != nil {
			v.Vs = make(lua_global.VersionList)
		}
	}
}

func (v *Versions) saveToCache(pluginName string) {
	if len(v.Vs) == 0 {
		return
	}
	cacheFilePath := filepath.Join(cnf.GetCacheDir(), pluginName)
	if content, err := json.MarshalIndent(v.Vs, "", "  "); err == nil {
		if len(content) > 10 {
			os.WriteFile(cacheFilePath, content, os.ModePerm)
		}
	}
}

func (v *Versions) GetSdkVersions() (vs lua_global.VersionList) {
	// load plugin.
	if err := v.loadPlugin(); err != nil {
		return
	}

	pluginName := GetConfItemFromLua(v.Lua.L, PluginName)
	v.loadFromCache(pluginName)
	// TODO: prequisites check

	vs = v.Vs
	if len(vs) > 0 {
		return
	}

	crawl := v.Lua.L.GetGlobal("crawl")
	if crawl == nil || crawl.Type() != lua.LTFunction {
		return
	}

	if err := v.Lua.L.CallByParam(lua.P{
		Fn:      crawl,
		NRet:    1,
		Protect: true,
	}); err != nil {
		return
	}

	result := v.Lua.L.Get(-1)

	userData, ok := result.(*lua.LUserData)
	if !ok {
		return
	}

	if vl, ok := userData.Value.(lua_global.VersionList); ok {
		v.Vs = vl
		v.saveToCache(pluginName)
	}
	return
}

func (v *Versions) GetSortedVersionList() (vs []table.Row) {
	if len(v.Vs) == 0 {
		v.GetSdkVersions()
	}
	for vName := range v.Vs {
		vs = append(vs, table.Row{
			vName,
		})
	}
	utils.SortVersions(vs)
	return
}

func (v *Versions) GetVersionByName(versionName string) (sv lua_global.SDKVersion) {
	if len(v.Vs) == 0 {
		v.GetSdkVersions()
	}
	return v.Vs[versionName]
}

func (v *Versions) GetInstallerConfig() (ic *lua_global.InstallerConfig) {
	if err := v.loadPlugin(); err != nil {
		return
	}

	ic = lua_global.GetInstallerConfig(v.Lua.L)
	return
}

func (v *Versions) CloseLua() {
	v.Lua.Close()
}
