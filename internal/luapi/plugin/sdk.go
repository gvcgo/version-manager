package plugin

import (
	"fmt"
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

// TODO: get and save sdk versions to cache files.
func (v *Versions) GetSdkVersions() (vs lua_global.VersionList) {
	vs = v.Vs
	if len(vs) > 0 {
		return
	}
	if err := v.loadPlugin(); err != nil {
		return
	}
	// TODO: prequisites check

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
