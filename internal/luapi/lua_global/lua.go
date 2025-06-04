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
	l.SetGlobal("vmrInitSelection", InitSelection)
	l.SetGlobal("vmrFind", Find)
	l.SetGlobal("vmrEq", Eq)
	l.SetGlobal("vmrAttr", Attr)
	l.SetGlobal("vmrText", Text)
	l.SetGlobal("vmrEach", Each)
	// gjson
	l.SetGlobal("vmrInitGJson", InitGJson)
	l.SetGlobal("vmrGetString", GetGJsonString)
	l.SetGlobal("vmrGetInt", GetGJsonInt)
	l.SetGlobal("vmrGetByKey", GetGJsonFromMapByKey) // for dict
	l.SetGlobal("vmrMapEach", GetGJsonMapEach)
	l.SetGlobal("vmrGetByIndex", GetGJsonFromSliceByIndex) // for array
	l.SetGlobal("vmrSliceEach", GetGJsonSliceEach)
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
	l.SetGlobal("vmrToLower", ToLower)
	l.SetGlobal("vmrSplit", Split)
	l.SetGlobal("vmrSprintf", Sprintf)
	l.SetGlobal("vmrUrlJoin", UrlJoin)
	l.SetGlobal("vmrLenString", LenString)
	l.SetGlobal("vmrGetOsEnv", GetOsEnv)
	l.SetGlobal("vmrSetOsEnv", SetOsEnv)
	l.SetGlobal("vmrExecSystemCmd", ExecSystemCmd)
	l.SetGlobal("vmrReadFile", ReadFile)
	l.SetGlobal("vmrWriteFile", WriteFile)
	l.SetGlobal("vmrCopyFile", CopyFile)
	l.SetGlobal("vmrCopyDir", CopyDir)
	// version
	l.SetGlobal("vmrNewVersionList", NewVersionList)
	l.SetGlobal("vmrAddItem", AddItem)
	l.SetGlobal("vmrMergeVersionList", MergeVersionList)
	// github
	l.SetGlobal("vmrGetGithubRelease", GetGithubRelease)
	// installer_config
	l.SetGlobal("vmrNewInstallerConfig", NewInstallerConf)
	l.SetGlobal("vmrAddFlagFiles", AddFlagFiles)
	l.SetGlobal("vmrEnableFlagDirExcepted", EnableFlagDirExcepted)
	l.SetGlobal("vmrAddBinaryDirs", AddBinaryDirs)
	l.SetGlobal("vmrAddAdditionalEnvs", AddAdditionalEnvs)
	// conda
	l.SetGlobal("vmrSearchByConda", SearchByConda)
}

func (l *Lua) GetLState() *lua.LState {
	return l.L
}
