package lua_global

import (
	lua "github.com/yuin/gopher-lua"
)

func InitLua() {
	L := lua.NewState()
	defer L.Close()
}

type Lua struct {
	L *lua.LState
}

func NewLua() *Lua {
	l := &Lua{
		L: lua.NewState(),
	}
	l.init()
	return l
}

func (l *Lua) Close() {
	l.L.Close()
}

func (l *Lua) setGlobal(name string, fn lua.LGFunction) {
	l.L.SetGlobal(name, l.L.NewFunction(fn))
}

func (l *Lua) init() {
	l.setGlobal("getResponse", GetResponse)
	// goquery
	l.setGlobal("initSelection", InitSelection)
	l.setGlobal("find", Find)
	l.setGlobal("eq", Eq)
	l.setGlobal("attr", Attr)
	l.setGlobal("text", Text)
	l.setGlobal("each", Each)
	// gjson
	l.setGlobal("initGJson", InitGJson)
	l.setGlobal("getString", GetGJsonString)
	l.setGlobal("getInt", GetGJsonInt)
	l.setGlobal("getByKey", GetGJsonFromMapByKey)
	l.setGlobal("mapEach", GetGJsonMapEach)
	l.setGlobal("getByIndex", GetGJsonFromSliceByIndex)
	l.setGlobal("sliceEach", GetGJsonSliceEach)
	// utils
	l.setGlobal("getOsArch", GetOsArch)
	l.setGlobal("regexpFindString", RegExpFindString)
}

func (l *Lua) GetLState() *lua.LState {
	return l.L
}
