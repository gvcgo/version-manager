package lua_global

import (
	"fmt"
	"net/url"
	"regexp"
	"runtime"
	"strings"

	lua "github.com/yuin/gopher-lua"
)

/*
lua: os, arch = vmrGetOsArch()
*/
func GetOsArch(L *lua.LState) int {
	L.Push(lua.LString(runtime.GOOS))
	L.Push(lua.LString(runtime.GOARCH))
	return 2
}

/*
lua: r = vmrRegExpFindString(pattern, string)
*/
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

/*
lua: bool = vmrHasPrefix(string, prefix)
*/
func HasPrefix(L *lua.LState) int {
	str := L.ToString(1)
	prefix := L.ToString(2)
	result := strings.HasPrefix(str, prefix)
	L.Push(lua.LBool(result))
	return 1
}

/*
lua: bool = vmrHasSuffix(string, suffix)
*/
func HasSuffix(L *lua.LState) int {
	str := L.ToString(1)
	suffix := L.ToString(2)
	result := strings.HasSuffix(str, suffix)
	L.Push(lua.LBool(result))
	return 1
}

/*
lua: bool = vmrContains(string, substring)
*/
func Contains(L *lua.LState) int {
	str := L.ToString(1)
	substr := L.ToString(2)
	result := strings.Contains(str, substr)
	L.Push(lua.LBool(result))
	return 1
}

/*
lua: s = vmrTrimPrefix(string, prefix)
*/
func TrimPrefix(L *lua.LState) int {
	str := L.ToString(1)
	prefix := L.ToString(2)
	result := strings.TrimPrefix(str, prefix)
	L.Push(lua.LString(result))
	return 1
}

/*
lua: s = vmrTrimSuffix(string, suffix)
*/
func TrimSuffix(L *lua.LState) int {
	str := L.ToString(1)
	suffix := L.ToString(2)
	result := strings.TrimSuffix(str, suffix)
	L.Push(lua.LString(result))
	return 1
}

/*
lua: s = vmrTrim(string, substring)
*/
func Trim(L *lua.LState) int {
	str := L.ToString(1)
	s := L.ToString(2)
	result := strings.Trim(str, s)
	L.Push(lua.LString(result))
	return 1
}

/*
lua: s = vmrTrimSpace(string)
*/
func TrimSpace(L *lua.LState) int {
	str := L.ToString(1)
	result := strings.TrimSpace(str)
	L.Push(lua.LString(result))
	return 1
}

/*
lua: string = vmrSprintf(pattern, {s1, s2, s3, ...})
*/
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

/*
lua: s = vmrUrlJoin(base, path)
*/
func UrlJoin(L *lua.LState) int {
	base := L.ToString(1)
	paths := L.ToString(2)
	result, _ := url.JoinPath(base, paths)
	L.Push(lua.LString(result))
	return 1
}

/*
lua: int = vmrLenString(string)
*/
func LenString(L *lua.LState) int {
	str := L.ToString(1)
	L.Push(lua.LNumber(len(str)))
	return 1
}
