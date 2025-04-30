package lua_global

import "testing"

var githubScript = `print("------------github------------")

function tagFilter(str)
	local s = regexpFindString("v\\d+(.\\d+){2}", str)
	if s ~= "" then
		return true
	end
	return false
end

function versionParser(str)
	local s = regexpFindString("v(\\d+)(.\\d+){2}", str)
	s = trimPrefix(s, "v")
	return s
end

function fileFilter(str)
	if contains(str, "profile") then
		return false
	end
	if contains(str, "baseline") then
		return false
	end
	if hasSuffix(str, ".txt") then
		return false
	end
	if hasSuffix(str, ".txt.asc") then
		return false
	end
	if hasSuffix(str, "musl.zip") then
		return false
	end
	return true
end

function archParser(str)
	if contains(str, "-x64") then
		return "amd64"
	end
	if contains(str, "-aarch64") then
		return "arm64"
	end
	return ""
end

function osParser(str)
	if contains(str, "linux") then
		return "linux"
	end
	if contains(str, "darwin") then
		return "darwin"
	end
	if contains(str, "windows") then
		return "windows"
	end
	return ""
end

function installerGetter(str)
	return "unarchiver"
end

local result = getGithubRelease("oven-sh/bun", tagFilter, versionParser, fileFilter, archParser, osParser, installerGetter)
print(result)
`

func TestGithub(t *testing.T) {
	ll := NewLua()
	defer ll.Close()
	L := ll.GetLState()

	if err := L.DoString(githubScript); err != nil {
		t.Error(err)
	}
}
