package lua_global

import (
	"fmt"
	"testing"
)

var condaScript = `print("-----------------conda-------------------")

local vl = newVersionList()
local result = searchByConda(vl, "php")

print(result)
`

func TestConda(t *testing.T) {
	fmt.Println("test conda")
	ll := NewLua()
	defer ll.Close()

	if err := ll.GetLState().DoString(condaScript); err != nil {
		t.Error(err)
	}
}
