package plugin

import (
	lua "github.com/yuin/gopher-lua"
)

type LuaConfItem string

const (
	SDKName       LuaConfItem = "sdk_name"
	PluginVersion LuaConfItem = "plugin_version"
	Prequisite    LuaConfItem = "prequisite"
	Homepage      LuaConfItem = "homepage"
)

func GetConfItemFromLua(L *lua.LState, item LuaConfItem) (result string) {
	v := L.GetGlobal(string(item))
	if v == nil {
		return
	}
	if v.Type() == lua.LTString {
		return v.String()
	} else if v.Type() == lua.LTFunction {
		if err := L.CallByParam(lua.P{
			Fn:      v,
			NRet:    1,
			Protect: true,
		}); err != nil {
			panic(err)
		}
		result := L.Get(-1)
		return result.String()
	}
	return
}
