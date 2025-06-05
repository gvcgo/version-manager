package lua_global

import (
	"net/url"

	"github.com/gvcgo/version-manager/internal/cnf"
	lua "github.com/yuin/gopher-lua"
)

/*
lua: scheme string, host string, port string = vmrGetProxy()
*/
func GetProxy(L *lua.LState) int {
	cfg := cnf.NewVMRConf()
	if cfg.ProxyUri == "" {
		L.Push(lua.LString(""))
		L.Push(lua.LString(""))
		L.Push(lua.LString("0"))
		return 3
	}

	parsedURL, err := url.Parse(cfg.ProxyUri)
	if err != nil || parsedURL == nil {
		L.Push(lua.LString(""))
		L.Push(lua.LString(""))
		L.Push(lua.LString("0"))
		return 3
	}

	L.Push(lua.LString(parsedURL.Scheme))
	L.Push(lua.LString(parsedURL.Hostname()))
	L.Push(lua.LString(parsedURL.Port()))
	return 3
}
