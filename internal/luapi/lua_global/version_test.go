package lua_global

import (
	"fmt"
	"testing"

	lua "github.com/yuin/gopher-lua"
)

func TestNewVersionList(t *testing.T) {
	script := `vl = vmrNewVersionList()
	print(vl)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func ExecuteLuaScriptL(script string) (*lua.LState, error) {
	ll := NewLua()
	L := ll.GetLState()
	err := L.DoString(script)
	return L, err
}

func TestAddItem(t *testing.T) {
	script := `vl = vmrNewVersionList()
	item = {
		["url"] = "https://xxx",
		["arch"] = "amd64",
		["os"] = "linux",
		["sum"] = "",
		["sum_type"] = "",
		["size"] = 10,
		["installer"] = "archiver",
		["lts"] = "",
		["extra"] = ""
	}
	vl = vmrAddItem(vl, "v0.0.1", item)
	print(vl)
	`
	if l, err := ExecuteLuaScriptL(script); err != nil {
		l.Close()
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

func TestMergeVersionList(t *testing.T) {
	script := `vl = vmrNewVersionList()
		item = {
			["url"] = "https://xxx",
			["arch"] = "amd64",
			["os"] = "linux",
			["sum"] = "",
			["sum_type"] = "",
			["size"] = 10,
			["installer"] = "archiver",
			["lts"] = "",
			["extra"] = ""
		}
		vmrAddItem(vl, "v0.0.1", item)

		vl2 = vmrNewVersionList()
		item = {
			["url"] = "https://yyy",
			["arch"] = "amd64",
			["os"] = "linux",
			["sum"] = "",
			["sum_type"] = "",
			["size"] = 10,
			["installer"] = "archiver",
			["lts"] = "",
			["extra"] = ""
		}
		vmrAddItem(vl2, "v0.0.2", item)

		vmrMergeVersionList(vl, vl2)
		print(vl)
		`
	if l, err := ExecuteLuaScriptL(script); err != nil {
		l.Close()
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
