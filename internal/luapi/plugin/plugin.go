package plugin

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
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
	VersionList lua_global.VersionList
}

type Plugin struct {
	FileName      string `json:"file_name"`
	FileContent   string `json:"file_content"`
	PluginName    string `json:"plugin_name"`
	PluginVersion string `json:"plugin_version"`
	SDKName       string `json:"sdk_name"`
	Homepage      string `json:"homepage"`
	result        Result
}

func NewPlugin(fileName string, fileContent ...string) (*Plugin, error) {
	var content string
	if len(fileContent) > 0 {
		content = fileContent[0]
	}
	p := &Plugin{
		FileName:    fileName,
		FileContent: content,
		result: Result{
			VersionList: make(map[string]lua_global.Item),
		},
	}
	if err := p.Load(); err != nil {
		return nil, err
	}
	return p, nil
}

func (p *Plugin) getPluginFilePath() string {
	pDir := cnf.GetPluginDir()
	return filepath.Join(pDir, p.FileName)
}

func (p *Plugin) LuaDo() error {
	if p.result.Lua == nil {
		p.result.Lua = lua_global.NewLua()
	}
	if p.FileName != "" {
		pluginPath := p.getPluginFilePath()
		if ok, _ := gutils.PathIsExist(pluginPath); !ok {
			return fmt.Errorf("plugin file not found: %s", pluginPath)
		}

		if err := p.result.Lua.L.DoFile(pluginPath); err != nil {
			return fmt.Errorf("failed to load plugin: %s", err)
		}
	} else if p.FileContent != "" {
		if err := p.result.Lua.L.DoString(p.FileContent); err != nil {
			return fmt.Errorf("failed to load plugin: %s", err)
		}
	}
	return nil
}

func (p *Plugin) Load() error {
	if err := p.LuaDo(); err != nil {
		return err
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
	if p.PluginName != "" {
		return filepath.Join(cnf.GetCacheDir(), p.PluginName)
	}
	return ""
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
	if cnf.GetCacheDisabled() {
		// cache is disabled.
		return
	}
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

func (p *Plugin) GetSDKVersions() (vl lua_global.VersionList, err error) {
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
		p.result.VersionList = vl
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

func (p *Plugin) getFuncFromLua(luaItem LuaConfItem, args ...string) CustomedFuncFromLua {
	luaFunc := p.result.Lua.L.GetGlobal(string(luaItem))
	if luaFunc == nil || luaFunc.Type() != lua.LTFunction {
		return nil
	}

	f := func() (string, error) {
		luaFuncArgs := make([]lua.LValue, len(args))
		for i, arg := range args {
			luaFuncArgs[i] = lua.LString(arg)
		}

		// if luaFuncArgs have more args than the lua function expects, the extra args will be ignored.
		if err := p.result.Lua.L.CallByParam(lua.P{
			Fn:      luaFunc,
			NRet:    1,
			Protect: true,
		}, luaFuncArgs...); err != nil {
			return "", err
		}

		result := p.result.Lua.L.Get(-1)
		if !CheckStatusOfCustomedFuncFromLua(result.String()) {
			return "", fmt.Errorf("execute customed function<%s> from lua failed", string(luaItem))
		}
		return result.String(), nil
	}
	return f
}

// user can custom his/her own install method in lua plugins.
func (p *Plugin) GetCustomedInstallHandler(args ...string) CustomedFuncFromLua {
	return p.getFuncFromLua(CustomedInstall, args...)
}

func (p *Plugin) GetPreInstallHandler(args ...string) CustomedFuncFromLua {
	return p.getFuncFromLua(PreInstall, args...)
}

func (p *Plugin) GetPostInstallHandler(args ...string) CustomedFuncFromLua {
	return p.getFuncFromLua(PostInstall, args...)
}

func (p *Plugin) GetCustomedUninstallHandler(args ...string) CustomedFuncFromLua {
	return p.getFuncFromLua(CustomedUninstall, args...)
}

func (p *Plugin) GetCustomedFileNameHandler(args ...string) CustomedFuncFromLua {
	return p.getFuncFromLua(CustomedFileName, args...)
}
