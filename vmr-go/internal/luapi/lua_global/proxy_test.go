package lua_global

import "testing"

func TestGetProxy(t *testing.T) {
	script := `
	scheme, host, port = vmrGetProxy()
	print(scheme)
	print(host)
	print(port)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}
