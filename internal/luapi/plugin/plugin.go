package plugin

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"time"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/gvcgo/version-manager/internal/tui/table"
	"github.com/gvcgo/version-manager/internal/utils"
	lua "github.com/yuin/gopher-lua"
)

type Result struct {
	Lua         *lua_global.Lua
	VersionList map[string]lua_global.Item
}

type Plugin struct {
	FileName      string `json:"file_name"`
	PluginName    string `json:"plugin_name"`
	PluginVersion string `json:"plugin_version"`
	SDKName       string `json:"sdk_name"`
	Prequisite    string `json:"prequisite"`
	Homepage      string `json:"homepage"`
	result        Result
}

func NewPlugin(fileName string) *Plugin {
	return &Plugin{
		FileName: fileName,
		result: Result{
			VersionList: make(map[string]lua_global.Item),
		},
	}
}

func (p *Plugin) getPluginFilePath() string {
	pDir := cnf.GetPluginDir()
	return filepath.Join(pDir, p.FileName)
}

func (p *Plugin) Load() error {
	pluginPath := p.getPluginFilePath()
	if ok, _ := gutils.PathIsExist(pluginPath); !ok {
		return fmt.Errorf("plugin file not found: %s", pluginPath)
	}
	if p.result.Lua == nil {
		p.result.Lua = lua_global.NewLua()
	}

	if err := p.result.Lua.L.DoFile(pluginPath); err != nil {
		return fmt.Errorf("failed to load plugin file: %s, %s", pluginPath, err)
	}

	L := p.result.Lua.L

	p.PluginName = GetLuaConfItemString(L, PluginName)
	if p.PluginName == "" {
		return fmt.Errorf("plugin name not defined")
	}

	p.PluginVersion = GetLuaConfItemString(L, PluginVersion)
	p.SDKName = GetLuaConfItemString(L, SDKName)
	if p.SDKName == "" {
		return fmt.Errorf("SDK name not defined")
	}
	p.Prequisite = GetLuaConfItemString(L, Prequisite)
	p.Homepage = GetLuaConfItemString(L, Homepage)
	if p.Homepage == "" {
		return fmt.Errorf("homepage not defined")
	}

	if !DoesLuaItemExist(L, InstallerConfig) {
		return fmt.Errorf("installer config not found")
	}

	if !DoesLuaItemExist(L, Crawler) {
		return fmt.Errorf("Crawler<function crawl> not found")
	}
	return nil
}

func (p *Plugin) cacheDir() string {
	if p.PluginName == "" {
		if err := p.Load(); err != nil {
			return ""
		}
	}
	return filepath.Join(cnf.GetCacheDir(), p.PluginName)
}

func (p *Plugin) cacheFilePathForVersionList() string {
	versionListCacheDir := p.cacheDir()
	if versionListCacheDir == "" {
		return ""
	}

	if ok, _ := gutils.PathIsExist(versionListCacheDir); !ok {
		os.MkdirAll(versionListCacheDir, os.ModePerm)
	} else {
		ss, err := os.Stat(versionListCacheDir)
		if err == nil && !ss.IsDir() {
			os.RemoveAll(versionListCacheDir)
			os.MkdirAll(versionListCacheDir, os.ModePerm)
		}
	}
	cacheFileName := fmt.Sprintf("%s.versions.json", p.PluginName)
	return filepath.Join(versionListCacheDir, cacheFileName)
}

func (p *Plugin) loadVersionListFromCache() {
	cachePath := p.cacheFilePathForVersionList()
	if ok, _ := gutils.PathIsExist(cachePath); !ok {
		return
	}
	lastModifiedTime := utils.GetFileLastModifiedTime(cachePath)
	timeLag := time.Now().Unix() - lastModifiedTime
	if timeLag > cnf.GetCacheRetentionTime() {
		return
	}
	if content, err := os.ReadFile(cachePath); err == nil {
		err = json.Unmarshal(content, &p.result.VersionList)
		if err != nil {
			p.result.VersionList = map[string]lua_global.Item{}
		}
	}
}

func (p *Plugin) saveVersionListToCache() {
	if len(p.result.VersionList) == 0 {
		return
	}
	cacheFilePath := p.cacheFilePathForVersionList()
	if content, err := json.MarshalIndent(p.result.VersionList, "", "  "); err == nil {
		if len(content) > 10 {
			os.WriteFile(cacheFilePath, content, os.ModePerm)
		}
	}
}

func (p *Plugin) GetSDKVersions() (vl map[string]lua_global.Item, err error) {
	if p.result.Lua == nil {
		if err = p.Load(); err != nil {
			return
		}
	}

	p.loadVersionListFromCache()
	vl = p.result.VersionList
	if len(vl) > 0 {
		return
	}

	crawl := p.result.Lua.L.GetGlobal(string(Crawler))
	if crawl == nil || crawl.Type() != lua.LTFunction {
		err = fmt.Errorf("invalid plugin: missing crawl function: %s", p.PluginName)
		return
	}

	if err = p.result.Lua.L.CallByParam(lua.P{
		Fn:      crawl,
		NRet:    1,
		Protect: true,
	}); err != nil {
		return
	}

	result := p.result.Lua.L.Get(-1)
	userData, ok := result.(*lua.LUserData)
	if !ok {
		return nil, fmt.Errorf("invalid return value for function crawl in plugin: %s", p.PluginName)
	}

	if vl, ok := userData.Value.(lua_global.VersionList); ok {
		for vName, vv := range vl {
			for _, ver := range vv {
				if ver.Os == runtime.GOOS && ver.Arch == runtime.GOARCH {
					p.result.VersionList[vName] = ver
				}
			}
		}
		p.saveVersionListToCache()
	}
	return p.result.VersionList, nil
}

func (p *Plugin) Close() {
	if p.result.Lua != nil {
		p.result.Lua.L.Close()
	}
}

func (p *Plugin) GetSortedVersions() (vt []table.Row) {
	if len(p.result.VersionList) == 0 {
		if _, err := p.GetSDKVersions(); err != nil {
			return
		}
	}
	for vName := range p.result.VersionList {
		vt = append(vt, table.Row{
			vName,
		})
	}
	utils.SortVersions(vt)
	return
}

func (p *Plugin) GetVersion(versionName string) (r lua_global.Item) {
	if len(p.result.VersionList) == 0 {
		if _, err := p.GetSDKVersions(); err != nil {
			return
		}
	}
	return p.result.VersionList[versionName]
}

func (p *Plugin) GetLatestVersion() (versionName string, r lua_global.Item) {
	vt := p.GetSortedVersions()
	if len(vt) == 0 {
		return
	}
	versionName = vt[0][0]
	r = p.GetVersion(versionName)
	return
}

func (p *Plugin) GetInstallerConfig() (ic *lua_global.InstallerConfig, err error) {
	if p.result.Lua == nil {
		if err = p.Load(); err != nil {
			return
		}
	}

	ic = lua_global.GetInstallerConfig(p.result.Lua.L)
	return
}

func (p *Plugin) GetSDKName() (sdkName string, err error) {
	if p.result.Lua == nil {
		if err = p.Load(); err != nil {
			return
		}
	}
	sdkName = p.SDKName
	return
}
