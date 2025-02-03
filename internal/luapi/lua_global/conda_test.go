package lua_global

import "testing"

var condaScript = `print("-----------------conda-------------------")

local vl = newVersionList()
local result = searchByConda(vl, "php")

print(result)
`

func TestConda(t *testing.T) {
	ll := NewLua()
	defer ll.Close()

	if err := ll.GetLState().DoString(condaScript); err != nil {
		t.Error(err)
	}
}
