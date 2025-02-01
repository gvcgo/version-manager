package luapi

import (
	"regexp"
	"runtime"

	lua "github.com/yuin/gopher-lua"
)

func GetOsArch(L *lua.LState) int {
	L.Push(lua.LString(runtime.GOOS))
	L.Push(lua.LString(runtime.GOARCH))
	return 2
}

func RegExpFindString(L *lua.LState) int {
	patternStr := L.ToString(1)
	content := L.ToString(2)
	if patternStr == "" || content == "" {
		return 0
	}

	re := regexp.MustCompile(patternStr)
	result := re.FindString(content)
	L.Push(lua.LString(result))
	return 1
}
