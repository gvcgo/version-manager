package lua_global

/*
Lua runtime.
*/
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

func (l *Lua) SetGlobal(name string, fn lua.LGFunction) {
	l.L.SetGlobal(name, l.L.NewFunction(fn))
}

func (l *Lua) init() {
	l.SetGlobal("getResponse", GetResponse)
	// goquery
	l.SetGlobal("initSelection", InitSelection)
	l.SetGlobal("find", Find)
	l.SetGlobal("eq", Eq)
	l.SetGlobal("attr", Attr)
	l.SetGlobal("text", Text)
	l.SetGlobal("each", Each)
	// gjson
	l.SetGlobal("initGJson", InitGJson)
	l.SetGlobal("getString", GetGJsonString)
	l.SetGlobal("getInt", GetGJsonInt)
	l.SetGlobal("getByKey", GetGJsonFromMapByKey) // for dict
	l.SetGlobal("mapEach", GetGJsonMapEach)
	l.SetGlobal("getByIndex", GetGJsonFromSliceByIndex) // for array
	l.SetGlobal("sliceEach", GetGJsonSliceEach)
	// utils
	l.SetGlobal("getOsArch", GetOsArch)
	l.SetGlobal("regexpFindString", RegExpFindString)
	l.SetGlobal("hasPrefix", HasPrefix)
	l.SetGlobal("hasSuffix", HasSuffix)
	l.SetGlobal("contains", Contains)
	l.SetGlobal("trimPrefix", TrimPrefix)
	l.SetGlobal("trimSuffix", TrimSuffix)
	l.SetGlobal("trim", Trim)
	l.SetGlobal("trimSpace", TrimSpace)
	l.SetGlobal("sprintf", Sprintf)
	l.SetGlobal("urlJoin", UrlJoin)
	l.SetGlobal("lenString", LenString)
	// version
	l.SetGlobal("newVersionList", NewVersionList)
	l.SetGlobal("addItem", AddItem)
	// github
	l.SetGlobal("getGithubRelease", GetGithubRelease)
	// installer_config
	l.SetGlobal("newInstallerConfig", NewInstallerConf)
	l.SetGlobal("addFlagFiles", AddFlagFiles)
	l.SetGlobal("enableFlagDirExcepted", EnableFlagDirExcepted)
	l.SetGlobal("addBinaryDirs", AddBinaryDirs)
	l.SetGlobal("addAdditionalEnvs", AddAdditionalEnvs)
}

func (l *Lua) GetLState() *lua.LState {
	return l.L
}
