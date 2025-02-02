package lua_global

import "testing"

var jsonScript = `local headers = {}
headers["User-Agent"] = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"
local url = "https://mirrors.tuna.tsinghua.edu.cn/julia-releases/bin/versions.json"

local resp = getResponse(url, 10,headers)
local j = initGJson(resp)

print("----------------gjson----------------")

function parseMapA(k, v)
	print("=======***")
	local jj = initGJson(v)
	
	local stable = getString(jj, "stable")
	if stable ~= "true" then
		return
	end

	function parseSlice(idx, vvv)
		local mapJ = initGJson(vvv)
		local item = {}
		item["version"] = getString(mapJ, "version")
		item["url"] = getString(mapJ, "url")
		item["sha256"] = getString(mapJ, "sha256")
		item["os"] = getString(mapJ, "os")
		item["arch"] = getString(mapJ, "arch")
		item["size"] = getInt(mapJ, "size")
		-- print(item.url)
	end
	sliceEach(jj, "files", parseSlice)
end

mapEach(j, ".", parseMapA)
`

func TestGJson(t *testing.T) {
	ll := NewLua()
	defer ll.Close()
	L := ll.GetLState()

	if err := L.DoString(jsonScript); err != nil {
		t.Error(err)
	}
}
