package lua_global

import (
	"fmt"
	"testing"

	lua "github.com/yuin/gopher-lua"
)

var githubScript = `
function tagFilter(str)
	local s = vmrRegexpFindString("v\\d+(.\\d+){2}", str)
	if s ~= "" then
		return true
	end
	return false
end

function versionParser(str)
	local s = vmrRegexpFindString("v(\\d+)(.\\d+){2}", str)
	s = vmrTrimPrefix(s, "v")
	return s
end

function fileFilter(str)
	if vmrContains(str, "profile") then
		return false
	end
	if vmrContains(str, "baseline") then
		return false
	end
	if vmrHasSuffix(str, ".txt") then
		return false
	end
	if vmrHasSuffix(str, ".txt.asc") then
		return false
	end
	if vmrHasSuffix(str, "musl.zip") then
		return false
	end
	return true
end

function archParser(str)
	if vmrContains(str, "-x64") then
		return "amd64"
	end
	if vmrContains(str, "-aarch64") then
		return "arm64"
	end
	return ""
end

function osParser(str)
	if vmrContains(str, "linux") then
		return "linux"
	end
	if vmrContains(str, "darwin") then
		return "darwin"
	end
	if vmrContains(str, "windows") then
		return "windows"
	end
	return ""
end

function installerGetter(str)
	return "unarchiver"
end

result = vmrGetGithubRelease("oven-sh/bun", tagFilter, versionParser, fileFilter, archParser, osParser, installerGetter)
print(result)
`

func TestGithub(t *testing.T) {
	l, err := ExecuteLuaScriptL(githubScript)
	if err != nil {
		t.Error(err)
	} else {
		v := l.GetGlobal("result")

		if v.Type() == lua.LTUserData {
			ud := v.(*lua.LUserData)
			if ud == nil {
				return
			}
			if vl, ok := ud.Value.(VersionList); ok {
				_, _ = fmt.Printf("version list: %+v\n", vl)
			}
		}
	}
}
