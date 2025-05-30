package plugin

import (
	"fmt"
	"testing"

	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
	"github.com/stretchr/testify/assert"
	lua "github.com/yuin/gopher-lua"
)

var pluginScript = `
sdk_name = "lua"
plugin_name = "lua"
plugin_version = "0.0.1"
prequisite = "miniconda"
homepage = "https://www.lua.org/"
function crawl()
    print("Crawling...")
end
`

func ExecuteLuaScriptL(script string) (*lua.LState, error) {
	ll := lua_global.NewLua()
	L := ll.GetLState()
	err := L.DoString(script)
	return L, err
}

func TestDoesLuaItemExist(t *testing.T) {
	if l, err := ExecuteLuaScriptL(pluginScript); err != nil {
		t.Error(err)
	} else {
		ok1 := DoesLuaItemExist(l, SDKName)
		assert.Equal(t, true, ok1, "should be true")
		ok2 := DoesLuaItemExist(l, LuaConfItem("none"))
		assert.Equal(t, false, ok2, "should be false")
	}
}

func TestGetLuaConfItem(t *testing.T) {
	if l, err := ExecuteLuaScriptL(pluginScript); err != nil {
		t.Error(err)
	} else {
		sdkName := GetLuaConfItemString(l, SDKName)
		sdkNameShouldBe := "lua"
		assert.Equal(t, sdkNameShouldBe, sdkName, fmt.Sprintf("sdk_name should be: %s", sdkNameShouldBe))
		s := GetLuaConfItemString(l, LuaConfItem("none"))
		noneShouldBe := ""
		assert.Equal(t, noneShouldBe, s, fmt.Sprintf("none should be: %s", noneShouldBe))
	}
}
