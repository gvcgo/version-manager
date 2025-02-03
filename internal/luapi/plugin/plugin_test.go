package plugin

import (
	"fmt"
	"testing"

	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	lua "github.com/yuin/gopher-lua"
)

var goPluginPath = "/Users/moqsien/projects/lua/vmr_plugins/go.lua"

func TestGoPlugin(t *testing.T) {
	ll := lua_global.NewLua()
	defer ll.L.Close()

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
		fmt.Println(vl["go1.23.5"][0].Extra)
	}
}
