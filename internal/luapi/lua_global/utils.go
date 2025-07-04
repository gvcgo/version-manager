package lua_global

import (
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/utils"
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
		L.Push(lua.LString(""))
		return 1
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
lua: s = vmrToLower(string)
*/
func ToLower(L *lua.LState) int {
	str := L.ToString(1)
	L.Push(lua.LString(strings.ToLower(str)))
	return 1
}

/*
lua: array, length = vmrSplit(string, sep)
*/
func Split(L *lua.LState) int {
	content := L.ToString(1)
	sep := L.ToString(2)

	if content == "" || sep == "" {
		L.Push(&lua.LTable{})
		L.Push(lua.LNumber(0))
		return 2
	}
	result := strings.Split(content, sep)
	table := &lua.LTable{}
	for _, item := range result {
		table.Append(lua.LString(item))
	}
	L.Push(table)
	L.Push(lua.LNumber(len(result)))
	return 2
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
lua: s = vmrPathJoin(base, path)
*/
func PathJoin(L *lua.LState) int {
	base := L.ToString(1)
	paths := L.ToString(2)
	if base == "" || paths == "" {
		L.Push(lua.LString(""))
		return 1
	}
	result := filepath.Join(base, paths)
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

/*
lua: string = vmrGetEnv(key string)
*/
func GetOsEnv(L *lua.LState) int {
	key := L.ToString(1)
	if key == "" {
		L.Push(lua.LString(""))
		return 1
	}
	L.Push(lua.LString(os.Getenv(key)))
	return 1
}

/*
lua: bool = vmrSetOsEnv()
*/
func SetOsEnv(L *lua.LState) int {
	key := L.ToString(1)
	value := L.ToString(2)
	err := os.Setenv(key, value)
	if err != nil {
		L.Push(lua.LFalse)
	} else {
		L.Push(lua.LTrue)
	}
	return 1
}

/*
lua: result string, ok bool = vmrExecSystemCmd(to_collect_output bool, workdir string, args {a, b, c, d, ...})
*/
func ExecSystemCmd(L *lua.LState) int {
	toCollectOutput := L.ToBool(1)
	workDir := L.ToString(2)

	args := make([]string, 0)
	array := L.ToTable(3)
	array.ForEach(func(l1, l2 lua.LValue) {
		args = append(args, l2.String())
	})

	runner := utils.NewSysCommandRunner(toCollectOutput, workDir, args...)
	err := runner.Run()
	result := runner.GetOutput()
	L.Push(lua.LString(result))
	if err != nil {
		L.Push(lua.LFalse)
	} else {
		L.Push(lua.LTrue)
	}

	return 2
}

/*
lua: string = vmrReadFile(filePath string)
*/
func ReadFile(L *lua.LState) int {
	filePath := L.ToString(1)
	if filePath == "" {
		L.Push(lua.LString(""))
		return 1
	}
	content, err := os.ReadFile(filePath)
	if err != nil {
		L.Push(lua.LString(""))
		return 1
	}
	L.Push(lua.LString(string(content)))
	return 1
}

/*
lua: bool = vmrWriteFile(filePath string, content string)
*/
func WriteFile(L *lua.LState) int {
	filePath := L.ToString(1)
	content := L.ToString(2)
	if len(content) == 0 {
		ud := L.ToUserData(2)
		if ud != nil {
			content = ud.Value.(string)
		}
	}
	if filePath == "" {
		L.Push(lua.LFalse)
		return 1
	}
	err := os.WriteFile(filePath, []byte(content), os.ModePerm)
	if err != nil {
		L.Push(lua.LFalse)
	} else {
		L.Push(lua.LTrue)
	}
	return 1
}

/*
lua: bool = vmrCopyFile(src string, dst string)
*/
func CopyFile(L *lua.LState) int {
	src := L.ToString(1)
	dst := L.ToString(2)
	_, err := utils.CopyFile(src, dst)
	if err != nil {
		L.Push(lua.LFalse)
	} else {
		L.Push(lua.LTrue)
	}
	return 1
}

/*
lua: bool = vmrCopyDir(src string, dst string)
*/
func CopyDir(L *lua.LState) int {
	src := L.ToString(1)
	dst := L.ToString(2)
	err := utils.CopyDirectory(src, dst)
	if err != nil {
		L.Push(lua.LFalse)
	} else {
		L.Push(lua.LTrue)
	}
	return 1
}

/*
lua: bool = vmrCreateDir(dirPath string)
*/
func CreateDir(L *lua.LState) int {
	dirPath := L.ToString(1)
	if dirPath == "" {
		L.Push(lua.LFalse)
		return 1
	}

	err := os.MkdirAll(dirPath, os.ModePerm)
	if err != nil {
		L.Push(lua.LFalse)
	} else {
		L.Push(lua.LTrue)
	}
	return 1
}

/*
lua: bool = vmrRemoveAll(dirPath string)
*/
func RemoveAll(L *lua.LState) int {
	dirPath := L.ToString(1)
	if dirPath == "" {
		L.Push(lua.LFalse)
		return 1
	}

	err := os.RemoveAll(dirPath)
	if err != nil {
		L.Push(lua.LFalse)
	} else {
		L.Push(lua.LTrue)
	}
	return 1
}

const (
	VerisonDirPattern        string = "%s%s"
	VersionDirSuffix         string = "_versions"
	VersionInstallDirPattern string = "%s-%s"
)

func GetInstallDir(sdkName, pluginName, version string) string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf(VerisonDirPattern, sdkName, VersionDirSuffix))
	os.MkdirAll(d, os.ModePerm)
	return filepath.Join(d, fmt.Sprintf(VersionInstallDirPattern, pluginName, version))
}

/*
lua: installationDir = vmrGetInstallationDir(sdkName string, pluginName string, version string)
*/
func GetInstallationDir(L *lua.LState) int {
	sdkName := L.ToString(1)
	pluginName := L.ToString(2)
	version := L.ToString(3)
	installDir := GetInstallDir(sdkName, pluginName, version)
	L.Push(lua.LString(installDir))
	return 1
}
