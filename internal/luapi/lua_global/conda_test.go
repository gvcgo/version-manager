package lua_global

import (
	"fmt"
	"testing"

	"github.com/gvcgo/version-manager/internal/utils"
	"github.com/stretchr/testify/assert"
	lua "github.com/yuin/gopher-lua"
)

func ExecuteLuaScript(script string) error {
	ll := NewLua()
	defer ll.Close()
	L := ll.GetLState()
	return L.DoString(script)
}

func TestConda(t *testing.T) {
	if !utils.IsMinicondaInstalled() {
		return
	}

	var condaScript = `
	vl = vmrNewVersionList()
	vl = vmrSearchByConda(vl, "php")
	print(vl)
	`
	if l, err := ExecuteLuaScriptL(condaScript); err != nil {
		t.Error(err)
	} else {
		defer l.Close()
		v := l.GetGlobal("vl")

		if v.Type() == lua.LTUserData {
			ud := v.(*lua.LUserData)
			if ud == nil {
				return
			}
			if vl, ok := ud.Value.(VersionList); ok {
				fmt.Println("versionList: ", vl)
			}
		}
	}
}

func TestParseArch(t *testing.T) {
	platforms := map[string]string{
		"linux-64":      "amd64",
		"win-64":        "amd64",
		"osx-64":        "amd64",
		"linux-aarch64": "arm64",
		"win-arm64":     "arm64",
		"osx-arm64":     "arm64",
		"osx":           "",
	}

	for k, v := range platforms {
		a := ParseArch(k)
		assert.Equal(t, v, a, "should be equal")
	}
}

func TestParseOS(t *testing.T) {
	platforms := map[string]string{
		"linux-64":      "linux",
		"win-64":        "windows",
		"osx-64":        "darwin",
		"linux-aarch64": "linux",
		"win-arm64":     "windows",
		"osx-arm64":     "darwin",
		"amd64":         "",
	}

	for k, v := range platforms {
		o := ParseOS(k)
		assert.Equal(t, v, o, "should be equal")
	}
}
