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

func (l *Lua) SetGlobalFn(name string, fn lua.LGFunction) {
	l.L.SetGlobal(name, l.L.NewFunction(fn))
}

func (l *Lua) SetGlobalString(name string, value string) {
	l.L.SetGlobal(name, lua.LString(value))
}

func (l *Lua) init() {
	l.SetGlobalFn("vmrGetResponse", GetResponse)
	// installer type
	l.SetGlobalString("vmrInstallerConda", InstallerConda)
	l.SetGlobalString("vmrInstallerCoursier", InstallerCoursier)
	l.SetGlobalString("vmrInstallerRustup", InstallerRustup)
	l.SetGlobalString("vmrInstallerUnarchiver", InstallerUnarchiver)
	l.SetGlobalString("vmrInstallerExecutable", InstallerExecutable)
	l.SetGlobalString("vmrInstallerDpkg", InstallerDpkg)
	l.SetGlobalString("vmrInstallerRpm", InstallerRpm)
	// goquery
	l.SetGlobalFn("vmrInitSelection", InitSelection)
	l.SetGlobalFn("vmrFind", Find)
	l.SetGlobalFn("vmrEq", Eq)
	l.SetGlobalFn("vmrAttr", Attr)
	l.SetGlobalFn("vmrText", Text)
	l.SetGlobalFn("vmrEach", Each)
	// gjson
	l.SetGlobalFn("vmrInitGJson", InitGJson)
	l.SetGlobalFn("vmrGetString", GetGJsonString)
	l.SetGlobalFn("vmrGetInt", GetGJsonInt)
	l.SetGlobalFn("vmrGetByKey", GetGJsonFromMapByKey) // for dict
	l.SetGlobalFn("vmrMapEach", GetGJsonMapEach)
	l.SetGlobalFn("vmrGetByIndex", GetGJsonFromSliceByIndex) // for array
	l.SetGlobalFn("vmrSliceEach", GetGJsonSliceEach)
	// utils
	l.SetGlobalFn("vmrGetOsArch", GetOsArch)
	l.SetGlobalFn("vmrRegexpFindString", RegExpFindString)
	l.SetGlobalFn("vmrHasPrefix", HasPrefix)
	l.SetGlobalFn("vmrHasSuffix", HasSuffix)
	l.SetGlobalFn("vmrContains", Contains)
	l.SetGlobalFn("vmrTrimPrefix", TrimPrefix)
	l.SetGlobalFn("vmrTrimSuffix", TrimSuffix)
	l.SetGlobalFn("vmrTrim", Trim)
	l.SetGlobalFn("vmrTrimSpace", TrimSpace)
	l.SetGlobalFn("vmrToLower", ToLower)
	l.SetGlobalFn("vmrSplit", Split)
	l.SetGlobalFn("vmrSprintf", Sprintf)
	l.SetGlobalFn("vmrUrlJoin", UrlJoin)
	l.SetGlobalFn("vmrPathJoin", PathJoin)
	l.SetGlobalFn("vmrLenString", LenString)
	l.SetGlobalFn("vmrGetOsEnv", GetOsEnv)
	l.SetGlobalFn("vmrSetOsEnv", SetOsEnv)
	l.SetGlobalFn("vmrExecSystemCmd", ExecSystemCmd)
	l.SetGlobalFn("vmrReadFile", ReadFile)
	l.SetGlobalFn("vmrWriteFile", WriteFile)
	l.SetGlobalFn("vmrCopyFile", CopyFile)
	l.SetGlobalFn("vmrCopyDir", CopyDir)
	l.SetGlobalFn("vmrCreateDir", CreateDir)
	l.SetGlobalFn("vmrRemoveAll", RemoveAll)
	// version
	l.SetGlobalFn("vmrNewVersionList", NewVersionList)
	l.SetGlobalFn("vmrAddItem", AddItem)
	l.SetGlobalFn("vmrMergeVersionList", MergeVersionList)
	// github
	l.SetGlobalFn("vmrGetGithubRelease", GetGithubRelease)
	// installer_config
	l.SetGlobalFn("vmrNewInstallerConfig", NewInstallerConf)
	l.SetGlobalFn("vmrAddFlagFiles", AddFlagFiles)
	l.SetGlobalFn("vmrEnableFlagDirExcepted", EnableFlagDirExcepted)
	l.SetGlobalFn("vmrAddBinaryDirs", AddBinaryDirs)
	l.SetGlobalFn("vmrAddAdditionalEnvs", AddAdditionalEnvs)
	// conda
	l.SetGlobalFn("vmrSearchByConda", SearchByConda)
	// proxy
	l.SetGlobalFn("vmrGetProxy", GetProxy)
	// extractor
	l.SetGlobalFn("vmrUnarchive", Unarchive)
	// installation dir
	l.SetGlobalFn("vmrGetInstallationDir", GetInstallationDir)
}

func (l *Lua) GetLState() *lua.LState {
	return l.L
}
