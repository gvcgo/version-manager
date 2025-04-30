package plugin

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"time"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
	lua "github.com/yuin/gopher-lua"
)

type PrequisiteHandler func() error

/*
SDK Versions
*/
type Versions struct {
	Lua                *lua_global.Lua
	PluginName         string
	SDKName            string
	PrequisiteHandlers map[string]PrequisiteHandler
	versionList        map[string]lua_global.Item
}

func NewVersions(pluginName string) (v *Versions) {
	return &Versions{
		PluginName:         pluginName,
		PrequisiteHandlers: make(map[string]PrequisiteHandler),
		versionList:        map[string]lua_global.Item{},
	}
}

func (v *Versions) RegisterPrequisiteHandler(prequisiteName string, handler PrequisiteHandler) {
	v.PrequisiteHandlers[prequisiteName] = handler
}

func (v *Versions) loadPlugin() error {
	if v.Lua == nil {
		v.Lua = lua_global.NewLua()
	}
	pDir := cnf.GetPluginDir()

	pls := NewPlugins()
	pls.LoadAll()

	p := pls.GetPlugin(v.PluginName)
	if p.FileName == "" {
		return fmt.Errorf("plugin not found: %s", v.PluginName)
	}

	fPath := filepath.Join(pDir, p.FileName)
	if ok, _ := gutils.PathIsExist(fPath); !ok {
		return fmt.Errorf("plugin file not found: %s", v.PluginName)
	}
	if err := v.Lua.L.DoFile(fPath); err != nil {
		return fmt.Errorf("failed to load plugin file: %s", err)
	}

	v.SDKName = GetConfItemFromLua(v.Lua.L, SDKName)
	return nil
}

func (v *Versions) getCacheFilePath(pluginName string) string {
	cacheDir := cnf.GetCacheDir()
	versionCacheDir := filepath.Join(cacheDir, pluginName)
	if ok, _ := gutils.PathIsExist(versionCacheDir); !ok {
		os.MkdirAll(versionCacheDir, os.ModePerm)
	} else {
		ss, err := os.Stat(versionCacheDir)
		if err == nil && !ss.IsDir() {
			os.RemoveAll(versionCacheDir)
			os.MkdirAll(versionCacheDir, os.ModePerm)
		}
	}
	return filepath.Join(versionCacheDir, fmt.Sprintf("%s.versions.json", pluginName))
}

func (v *Versions) loadFromCache(pluginName string) {
	cacheFilePath := v.getCacheFilePath(pluginName)
	if ok, _ := gutils.PathIsExist(cacheFilePath); !ok {
		return
	}
	lastModifiedTime := utils.GetFileLastModifiedTime(cacheFilePath)
	timeLag := time.Now().Unix() - lastModifiedTime
	if timeLag > cnf.GetCacheRetentionTime() {
		return
	}
	if content, err := os.ReadFile(cacheFilePath); err == nil {
		err = json.Unmarshal(content, &v.versionList)
		if err != nil {
			v.versionList = map[string]lua_global.Item{}
		}
	}
}

func (v *Versions) saveToCache(pluginName string) {
	if len(v.versionList) == 0 {
		return
	}
	cacheFilePath := v.getCacheFilePath(pluginName)
	if content, err := json.MarshalIndent(v.versionList, "", "  "); err == nil {
		if len(content) > 10 {
			os.WriteFile(cacheFilePath, content, os.ModePerm)
		}
	}
}

func (v *Versions) GetSdkVersions() (vs map[string]lua_global.Item) {
	// load plugin.
	if err := v.loadPlugin(); err != nil {
		gprint.PrintError("load plugin failed: %s", v.PluginName)
		return
	}

	pluginName := GetConfItemFromLua(v.Lua.L, PluginName)
	v.loadFromCache(pluginName)

	vs = v.versionList
	if len(vs) > 0 {
		return
	}

	// prequisite := GetConfItemFromLua(v.Lua.L, Prequisite)
	// if prequisite != "" {
	// 	handler, ok := v.PrequisiteHandlers[prequisite]
	// 	if ok && handler != nil {
	// 		if err := handler(); err != nil {
	// 			gprint.PrintError("handle prequisite failed: %s", err)
	// 		}
	// 	}
	// }

	crawl := v.Lua.L.GetGlobal("crawl")
	if crawl == nil || crawl.Type() != lua.LTFunction {
		gprint.PrintError("invalid plugin: missing crawl function: %s", v.PluginName)
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
		for vName, vv := range vl {
			for _, ver := range vv {
				if ver.Os == runtime.GOOS && ver.Arch == runtime.GOARCH {
					v.versionList[vName] = ver
				}
			}
		}
		v.saveToCache(pluginName)
	}
	return
}

func (v *Versions) GetSortedVersionList() (vs []table.Row) {
	if len(v.versionList) == 0 {
		v.GetSdkVersions()
	}
	for vName := range v.versionList {
		vs = append(vs, table.Row{
			vName,
		})
	}
	utils.SortVersions(vs)
	return
}

func (v *Versions) GetLatestVersion() (versionName string, r lua_global.Item) {
	vs := v.GetSortedVersionList()
	if len(vs) == 0 {
		return
	}
	versionName = vs[0][0]
	r = v.GetVersionByName(versionName)
	return
}

func (v *Versions) GetVersionByName(versionName string) (r lua_global.Item) {
	if len(v.versionList) == 0 {
		v.GetSdkVersions()
	}
	return v.versionList[versionName]
}

func (v *Versions) GetInstallerConfig() (ic *lua_global.InstallerConfig) {
	if err := v.loadPlugin(); err != nil {
		return
	}

	ic = lua_global.GetInstallerConfig(v.Lua.L)
	return
}

func (v *Versions) GetSDKName() string {
	if err := v.loadPlugin(); err != nil {
		return ""
	}
	return v.SDKName
}

func (v *Versions) CloseLua() {
	v.Lua.Close()
}
