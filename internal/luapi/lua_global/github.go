package lua_global

import (
	"runtime"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global/gh"
	lua "github.com/yuin/gopher-lua"
)

func githubBoolFuncCall(L *lua.LState, cb *lua.LFunction, arg string) bool {
	defer func() {
		if err := recover(); err != nil {
			L.Push(lua.LBool(false))
		}
	}()
	if err := L.CallByParam(lua.P{
		Fn:      cb,
		NRet:    1,
		Protect: true,
	}, lua.LString(arg)); err != nil {
		panic(err)
	}
	result := L.Get(-1)
	if result.Type() == lua.LTBool {
		return gconv.Bool(result.String())
	}
	return false
}

func githubStringFuncCall(L *lua.LState, cb *lua.LFunction, arg string) string {
	defer func() {
		if err := recover(); err != nil {
			L.Push(lua.LString(""))
		}
	}()
	if err := L.CallByParam(lua.P{
		Fn:      cb,
		NRet:    1,
		Protect: true,
	}, lua.LString(arg)); err != nil {
		panic(err)
	}
	result := L.Get(-1)
	if result.Type() == lua.LTString {
		return result.String()
	}
	return ""
}

/*
lua:
vl = vmrNewVersionList()
vl = vmrGetGithubRelease(repoNameStr, tagFilterFunc, versionParserFunc, fileFilterFunc, archParserFunc, osParserFunc, installerGetterFunc)
*/
func GetGithubRelease(L *lua.LState) int {
	repoName := L.ToString(1)

	result := VersionList{}
	if repoName == "" {
		ud := L.NewUserData()
		ud.Value = result
		L.Push(ud)
		return 1
	}
	cfg := cnf.NewVMRConf()
	cfg.Load()

	if cfg.GithubToken == "" {
		cfg.GithubToken = gh.GetDefaultReadOnly()
	}

	client := gh.NewGh(repoName, cfg.GithubToken, cfg.ProxyUri, cfg.ReverseProxy)
	rl := client.GetReleases()

	tagFilter := L.ToFunction(2)
	versionParser := L.ToFunction(3)
	fileFilter := L.ToFunction(4)
	archParser := L.ToFunction(5)
	osParser := L.ToFunction(6)
	installerGetter := L.ToFunction(7)

	for _, rItem := range rl {
		if !githubBoolFuncCall(L, tagFilter, rItem.TagName) {
			continue
		}

		vStr := githubStringFuncCall(L, versionParser, rItem.TagName)
		if vStr == "" {
			continue
		}

	INNER:
		for _, a := range rItem.Assets {
			if strings.Contains(a.Url, "archive/refs/") {
				continue INNER
			}
			if !githubBoolFuncCall(L, fileFilter, a.Name) {
				continue INNER
			}
			item := Item{}
			item.Arch = githubStringFuncCall(L, archParser, a.Name)
			item.Os = githubStringFuncCall(L, osParser, a.Name)
			if item.Arch != runtime.GOARCH || item.Os != runtime.GOOS {
				continue INNER
			}

			item.Installer = githubStringFuncCall(L, installerGetter, a.Name)
			item.Url = a.Url
			item.Size = a.Size
			item.CreatedAt = a.CreatedAt.Unix()

			result[vStr] = item
		}
	}

	ud := L.NewUserData()
	ud.Value = result
	L.Push(ud)
	return 1
}
