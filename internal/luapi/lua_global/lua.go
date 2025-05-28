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
	l.SetGlobal("vmrGetResponse", GetResponse)
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
	l.SetGlobal("vmrGetOsArch", GetOsArch)
	l.SetGlobal("vmrRegexpFindString", RegExpFindString)
	l.SetGlobal("vmrHasPrefix", HasPrefix)
	l.SetGlobal("vmrHasSuffix", HasSuffix)
	l.SetGlobal("vmrContains", Contains)
	l.SetGlobal("vmrTrimPrefix", TrimPrefix)
	l.SetGlobal("vmrTrimSuffix", TrimSuffix)
	l.SetGlobal("vmrTrim", Trim)
	l.SetGlobal("vmrTrimSpace", TrimSpace)
	l.SetGlobal("vmrSprintf", Sprintf)
	l.SetGlobal("vmrUrlJoin", UrlJoin)
	l.SetGlobal("vmrLenString", LenString)
	// version
	l.SetGlobal("vmrNewVersionList", NewVersionList)
	l.SetGlobal("vmrAddItem", AddItem)
	l.SetGlobal("vmrMergeVersionList", MergeVersionList)
	// github
	l.SetGlobal("getGithubRelease", GetGithubRelease)
	// installer_config
	l.SetGlobal("newInstallerConfig", NewInstallerConf)
	l.SetGlobal("addFlagFiles", AddFlagFiles)
	l.SetGlobal("enableFlagDirExcepted", EnableFlagDirExcepted)
	l.SetGlobal("addBinaryDirs", AddBinaryDirs)
	l.SetGlobal("addAdditionalEnvs", AddAdditionalEnvs)
	// conda
	l.SetGlobal("vmrSearchByConda", SearchByConda)
}

func (l *Lua) GetLState() *lua.LState {
	return l.L
}
