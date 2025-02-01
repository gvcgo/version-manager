package luapi

import (
	"os"
	"time"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/request"
	lua "github.com/yuin/gopher-lua"
)

func prepareResult(L *lua.LState, result interface{}) {
	ud := L.NewUserData()
	ud.Value = result
	L.Push(ud)
}

const (
	ProxyEnvName string = "VCOLLECTOR_PROXY"
)

func GetResponse(L *lua.LState) int {
	dUrl := L.ToString(1)
	timeout := L.ToInt(2)
	headers := make(map[string]string)

	hTable := L.ToTable(3)

	if hTable != nil {
		hTable.ForEach(func(k lua.LValue, v lua.LValue) {
			headers[k.String()] = v.String()
		})
	}

	t := 180
	if timeout > 0 {
		t = timeout
	}

	fetcher := request.NewFetcher()
	fetcher.Headers = headers
	fetcher.SetUrl(dUrl)
	fetcher.Timeout = time.Duration(t) * time.Second

	proxy := os.Getenv(ProxyEnvName)
	// if gconv.Bool(proxy) && !strings.Contains(dUrl, "maven") && !strings.Contains(dUrl, "android") {
	if gconv.Bool(proxy) {
		fetcher.Proxy = proxy
	}
	// fetcher.Proxy = "http://127.0.0.1:2023"

	resp, code := fetcher.GetString()
	if code != 200 || resp == "" {
		return 0
	}
	prepareResult(L, resp)
	return 1
}
