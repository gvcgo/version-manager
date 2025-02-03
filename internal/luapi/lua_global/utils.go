package lua_global

import (
	"fmt"
	"regexp"
	"runtime"
	"strings"

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

func HasPrefix(L *lua.LState) int {
	str := L.ToString(1)
	prefix := L.ToString(2)
	result := strings.HasPrefix(str, prefix)
	L.Push(lua.LBool(result))
	return 1
}

func HasSuffix(L *lua.LState) int {
	str := L.ToString(1)
	suffix := L.ToString(2)
	result := strings.HasSuffix(str, suffix)
	L.Push(lua.LBool(result))
	return 1
}

func Contains(L *lua.LState) int {
	str := L.ToString(1)
	substr := L.ToString(2)
	result := strings.Contains(str, substr)
	L.Push(lua.LBool(result))
	return 1
}

func TrimPrefix(L *lua.LState) int {
	str := L.ToString(1)
	prefix := L.ToString(2)
	result := strings.TrimPrefix(str, prefix)
	L.Push(lua.LString(result))
	return 1
}

func TrimSuffix(L *lua.LState) int {
	str := L.ToString(1)
	suffix := L.ToString(2)
	result := strings.TrimSuffix(str, suffix)
	L.Push(lua.LString(result))
	return 1
}

func Trim(L *lua.LState) int {
	str := L.ToString(1)
	s := L.ToString(2)
	result := strings.Trim(str, s)
	L.Push(lua.LString(result))
	return 1
}

func Sprintf(L *lua.LState) int {
	pattern := L.ToString(1)
	array := L.ToTable(2)

	args := make([]any, 0)
	array.ForEach(func(l1, l2 lua.LValue) {
		args = append(args, l2.String())
	})
	result := fmt.Sprintf(pattern, args...)
	L.Push(lua.LString(result))
	return 1
}
