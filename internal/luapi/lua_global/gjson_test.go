package lua_global

import (
	"fmt"
	"testing"

	lua "github.com/yuin/gopher-lua"
)

var jsonScript = `local headers = {}
headers["User-Agent"] = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"
local url = "https://mirrors.tuna.tsinghua.edu.cn/julia-releases/bin/versions.json"

local resp = vmrGetResponse(url, 10,headers)
local j = vmrInitGJson(resp)
vl = vmrNewVersionList()

function parseMapA(k, v)
	local jj = vmrInitGJson(v)

	local stable = vmrGetString(jj, "stable")
	if stable ~= "true" then
		return
	end

	function parseSlice(idx, vvv)
		local mapJ = vmrInitGJson(vvv)
		item = {}
		item["version"] = vmrGetString(mapJ, "version")
		item["url"] = vmrGetString(mapJ, "url")
		item["sha256"] = vmrGetString(mapJ, "sha256")
		item["os"] = vmrGetString(mapJ, "os")
		item["arch"] = vmrGetString(mapJ, "arch")
		item["size"] = vmrGetInt(mapJ, "size")
		vl = vmrAddItem(vl, item.version,item)
	end
	vmrSliceEach(jj, "files", parseSlice)
end

vmrMapEach(j, ".", parseMapA)
print(vl)
`

func TestGJson(t *testing.T) {
	ll := NewLua()
	defer ll.Close()
	L := ll.GetLState()

	if err := L.DoString(jsonScript); err != nil {
		t.Error(err)
	}
	if l, err := ExecuteLuaScriptL(jsonScript); err != nil {
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
				_, _ = fmt.Printf("versionList: %+v", vl)
			}
		}
	}
}
