package lua_global

import "testing"

func TestGetOsArch(t *testing.T) {
	script := `os, arch = vmrGetOsArch()
	print(os)
	print(arch)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestRegExpFindString(t *testing.T) {
	script := `s = vmrRegexpFindString("r(.+)d", "hello regexp world")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestHasPrefix(t *testing.T) {
	script := `s = vmrHasPrefix("hello", "he")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestHasSuffix(t *testing.T) {
	script := `s = vmrHasSuffix("hello", "lo")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestContains(t *testing.T) {
	script := `if vmrContains("abc", "a") then
		print("true")
	end
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestTrimPrefix(t *testing.T) {
	script := `s = vmrTrimPrefix("abc", "a")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestTrimSuffix(t *testing.T) {
	script := `s = vmrTrimSuffix("abc", "c")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestTrim(t *testing.T) {
	script := `s = vmrTrim("dabcd", "d")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestTrimSpace(t *testing.T) {
	script := `s = vmrTrimSpace(" abc ")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestToLower(t *testing.T) {
	script := `s = vmrToLower("ABC")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestSplit(t *testing.T) {
	script := `list, len = vmrSplit("a,b,c", ",")
	print(list[1])
	print(len)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestSprintf(t *testing.T) {
	script := `s = vmrSprintf("abc %s", {"def"})
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestUrlJoin(t *testing.T) {
	script := `s = vmrUrlJoin("https://test.com/v1", "check")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestLenString(t *testing.T) {
	script := `s = vmrLenString("check")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestExecSystemCmd(t *testing.T) {
	script := `
	result, ok = vmrExecSystemCmd(true, "/home", {"ls", "-ahl"})
	print(result)
	print(ok)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}

func TestPathJoin(t *testing.T) {
	script := `s = vmrPathJoin("/home", "test")
	print(s)
	`
	if err := ExecuteLuaScript(script); err != nil {
		t.Error(err)
	}
}
