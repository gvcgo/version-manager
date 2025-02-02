package lua_global

import (
	"fmt"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global/gh"
	lua "github.com/yuin/gopher-lua"
)

func GetGithubRelease(L *lua.LState) int {
	repoName := L.ToString(1)
	if repoName == "" {
		return 0
	}
	cfg := cnf.NewVMRConf()
	cfg.Load()

	client := gh.NewGh(repoName, cfg.GithubToken, cfg.ProxyUri, cfg.ReverseProxy)
	rl := client.GetReleases()
	for _, r := range rl {
		// TODO: parse release item.
		fmt.Println(r)
	}
	return 1
}
