package lua_global

import (
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global/gh"
	lua "github.com/yuin/gopher-lua"
)

func githubBoolFuncCall(L *lua.LState, cb *lua.LFunction, arg string) bool {
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

func GetGithubRelease(L *lua.LState) int {
	repoName := L.ToString(1)
	if repoName == "" {
		return 0
	}
	cfg := cnf.NewVMRConf()
	cfg.Load()

	client := gh.NewGh(repoName, cfg.GithubToken, cfg.ProxyUri, cfg.ReverseProxy)
	rl := client.GetReleases()

	tagFilter := L.ToFunction(2)
	versionParser := L.ToFunction(3)
	fileFilter := L.ToFunction(4)
	archParser := L.ToFunction(5)
	osParser := L.ToFunction(6)
	installerGetter := L.ToFunction(7)

	result := make(VersionList)

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
			if item.Arch == "" || item.Os == "" {
				continue INNER
			}
			item.Installer = githubStringFuncCall(L, installerGetter, a.Name)
			item.Url = a.Url
			item.Size = a.Size

			if _, ok := result[vStr]; !ok {
				result[vStr] = SDKVersion{}
			}
			result[vStr] = append(result[vStr], item)
		}
	}

	// fmt.Println("******$$$$$$$", result["1.2.2"])
	ud := L.NewUserData()
	ud.Value = result
	L.Push(ud)
	return 1
}
