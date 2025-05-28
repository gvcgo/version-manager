package lua_global

import "testing"

func TestGetResponse(t *testing.T) {
	script := `url = "https://www.baidu.com/"
	timeout = 10
	headers = { ["User-Agent"] ="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"}
	resp = vmrGetResponse(url, timeout, headers)
	print(resp)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}
