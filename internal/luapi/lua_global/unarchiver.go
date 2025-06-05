package lua_global

import (
	"github.com/gvcgo/version-manager/internal/utils/extract"
	lua "github.com/yuin/gopher-lua"
)

/*
lua:
ok bool = vmrUnarchive(src string, dst string, compressedSingleFileName string, isCompressedSingleExecutable bool)
*/
func Unarchive(L *lua.LState) int {
	src := L.ToString(1)
	dstDir := L.ToString(2)
	if src == "" || dstDir == "" {
		L.Push(lua.LFalse)
		return 1
	}
	unarchiver := extract.New(src, dstDir)

	compressedSingleFileName := L.ToString(3)
	if compressedSingleFileName != "" {
		unarchiver.SetCompressedSingleFileName(compressedSingleFileName)
	}
	isCompressedSingleExecutable := L.ToBool(4)
	if isCompressedSingleExecutable {
		unarchiver.SetCompressedSingleExe()
	}

	err := unarchiver.Unarchive()
	if err != nil {
		L.Push(lua.LFalse)
	} else {
		L.Push(lua.LTrue)
	}
	return 1
}
