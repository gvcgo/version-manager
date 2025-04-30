package plugin

import (
	"fmt"
	"testing"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	lua "github.com/yuin/gopher-lua"
)

var goPluginPath = "/Users/moqsien/projects/lua/vmr_plugins/go.lua"
var minicondaPluginPath = "/Users/moqsien/projects/lua/vmr_plugins/miniconda.lua"
var coursierPluginPath = "/Users/moqsien/projects/lua/vmr_plugins/coursier.lua"
var luaPluginPath = "/Users/moqsien/projects/lua/vmr_plugins/lua.lua"

func TestGoPlugin(t *testing.T) {
	fmt.Println("aaa")

	ll := lua_global.NewLua()
	defer ll.L.Close()

	if ok, _ := gutils.PathIsExist(goPluginPath); !ok {
		return
	}

	if err := ll.L.DoFile(goPluginPath); err != nil {
		t.Error(err)
	}

	L := ll.GetLState()
	if sdkName := GetConfItemFromLua(L, SDKName); sdkName != "go" {
		t.Errorf("sdk_name should be 'go', but got '%s'", sdkName)
	}

	if pVersion := GetConfItemFromLua(L, PluginVersion); pVersion != "0.1" {
		t.Errorf("plugin_version should be '0.1', but got '%s'", pVersion)
	}

	if pre := GetConfItemFromLua(L, Prequisite); pre != "" {
		t.Errorf("prequisite should be '', but got '%s'", pre)
	}

	if hp := GetConfItemFromLua(L, Homepage); hp != "https://go.dev/" {
		t.Errorf("homepage should be 'https://go.dev/', but got '%s'", hp)
	}

	f := L.GetGlobal("crawl")
	if f == nil || f.Type() != lua.LTFunction {
		t.Error("crawl function should be defined")
	}

	if err := L.CallByParam(lua.P{
		Fn:      f,
		NRet:    1,
		Protect: true,
	}); err != nil {
		t.Error(err)
	}

	r := L.Get(-1)

	ud, ok := r.(*lua.LUserData)
	if !ok {
		t.Error("return value should be userdata")
	}

	if vl, ok := ud.Value.(lua_global.VersionList); !ok {
		t.Error("userdata value should be VersionList")
	} else {
		// keys := []string{}
		// for k := range vl {
		// 	keys = append(keys, k)
		// }
		// fmt.Println(keys)
		fmt.Println(vl["1.23.5"][0].Extra)
	}

	ic := lua_global.GetInstallerConfig(L)
	if ic == nil {
		t.Error("installer config should be defined")
		return
	}
	fmt.Println(ic.FlagFiles)
	fmt.Println(ic.BinaryDirs)
	fmt.Println(ic.AdditionalEnvs)
}

func TestMinicondaPlugin(t *testing.T) {
	fmt.Println("xxx-----")
	ll := lua_global.NewLua()
	defer ll.L.Close()
	if ok, _ := gutils.PathIsExist(minicondaPluginPath); !ok {
		return
	}

	if err := ll.L.DoFile(minicondaPluginPath); err != nil {
		t.Error(err)
	}

	L := ll.GetLState()

	if pluginName := GetConfItemFromLua(L, PluginName); pluginName != "miniconda" {
		t.Errorf("plugin_name should be 'miniconda', but got '%s'", pluginName)
	}

	f := L.GetGlobal("crawl")
	if f == nil || f.Type() != lua.LTFunction {
		t.Error("crawl function should be defined")
	}

	if err := L.CallByParam(lua.P{
		Fn:      f,
		NRet:    1,
		Protect: true,
	}); err != nil {
		t.Error(err)
	}

	r := L.Get(-1)

	ud, ok := r.(*lua.LUserData)
	if !ok {
		t.Error("return value should be userdata")
	}

	if vl, ok := ud.Value.(lua_global.VersionList); !ok {
		t.Error("userdata value should be VersionList")
	} else {
		// keys := []string{}
		// for k := range vl {
		// 	keys = append(keys, k)
		// }
		// fmt.Println(keys)
		fmt.Println(vl["latest"])
	}

	ic := lua_global.GetInstallerConfig(L)
	if ic == nil {
		t.Error("installer config should be defined")
		return
	}
	fmt.Println(ic.BinaryDirs)
}

func TestCoursierPlugin(t *testing.T) {
	fmt.Println("test coursier plugin")

	ll := lua_global.NewLua()
	defer ll.L.Close()
	if ok, _ := gutils.PathIsExist(coursierPluginPath); !ok {
		return
	}

	if err := ll.L.DoFile(coursierPluginPath); err != nil {
		t.Error(err)
	}

	L := ll.GetLState()

	if pluginName := GetConfItemFromLua(L, PluginName); pluginName != "coursier" {
		t.Errorf("plugin_name should be 'coursier', but got '%s'", pluginName)
	}

	f := L.GetGlobal("crawl")
	if f == nil || f.Type() != lua.LTFunction {
		t.Error("crawl function should be defined")
	}

	if err := L.CallByParam(lua.P{
		Fn:      f,
		NRet:    1,
		Protect: true,
	}); err != nil {
		t.Error(err)
	}

	r := L.Get(-1)

	ud, ok := r.(*lua.LUserData)
	if !ok {
		t.Error("return value should be userdata")
	}

	if vl, ok := ud.Value.(lua_global.VersionList); !ok {
		t.Error("userdata value should be VersionList")
	} else {
		// keys := []string{}
		// for k := range vl {
		// 	keys = append(keys, k)
		// }
		// fmt.Println(keys)
		fmt.Println(vl["2.1.24"])
	}

	ic := lua_global.GetInstallerConfig(L)
	if ic == nil {
		t.Error("installer config should be defined")
		return
	}
	fmt.Println(ic.FlagFiles)
}

func TestLuaPlugin(t *testing.T) {
	fmt.Println("test lua plugin")

	ll := lua_global.NewLua()
	defer ll.L.Close()
	if ok, _ := gutils.PathIsExist(luaPluginPath); !ok {
		return
	}

	if err := ll.L.DoFile(luaPluginPath); err != nil {
		t.Error(err)
	}

	L := ll.GetLState()

	if pre := GetConfItemFromLua(L, Prequisite); pre != "conda" {
		t.Errorf("prequisite should be 'conda', but got '%s'", pre)
	}

	f := L.GetGlobal("crawl")
	if f == nil || f.Type() != lua.LTFunction {
		t.Error("crawl function should be defined")
	}

	if err := L.CallByParam(lua.P{
		Fn:      f,
		NRet:    1,
		Protect: true,
	}); err != nil {
		t.Error(err)
	}

	r := L.Get(-1)

	ud, ok := r.(*lua.LUserData)
	if !ok {
		t.Error("return value should be userdata")
	}

	if vl, ok := ud.Value.(lua_global.VersionList); !ok {
		t.Error("userdata value should be VersionList")
	} else {
		// keys := []string{}
		// for k := range vl {
		// 	keys = append(keys, k)
		// }
		// fmt.Println(keys)
		fmt.Println(vl["5.4.6"])
	}

	ic := lua_global.GetInstallerConfig(L)
	if ic == nil {
		t.Error("installer config should be defined")
		return
	}
	fmt.Println(ic.BinaryDirs)
}

func TestPluginsLoadAll(t *testing.T) {
	fmt.Println("test plugins load all aaaaa")
	p := NewPlugins()
	p.LoadAll()
	fmt.Println(p.pls)
}
