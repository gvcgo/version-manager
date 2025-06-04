package lua_global

import "testing"

var goqueryScript = `local headers = {}
headers["User-Agent"] = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"

local resp = vmrGetResponse("https://www.bing.com", 10, headers)
local s = vmrInitSelection(resp, "li")

function parseLiItem(i, ss)
    local node = vmrFind(ss, "a")
    local text = vmrText(node)
    local href = vmrAttr(node, "href")
    local s = vmrSprintf("%s: %s", {text, href})
    print(s)
end
vmrEach(s, parseLiItem)
`

func TestGoQuery(t *testing.T) {
	ll := NewLua()
	defer ll.Close()
	L := ll.GetLState()

	if err := L.DoString(goqueryScript); err != nil {
		t.Error(err)
	}
}
