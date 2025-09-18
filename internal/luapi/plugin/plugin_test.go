package plugin

import (
	"fmt"
	"os"
	"testing"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/stretchr/testify/assert"
)

var pluginForGo = `--[[
    Go language support for VMR.
--]]

-- global variables
sdk_name = "go"
plugin_name = "go"
plugin_version = "0.1"
prequisite = ""
homepage = "https://go.dev/"

-- installer config
ic = vmrNewInstallerConfig()
ic = vmrAddFlagFiles(ic , "", {"VERSION", "LICENSE"})
ic = vmrAddBinaryDirs(ic, "", {"bin"})
ic = vmrAddAdditionalEnvs(ic , "GOROOT", {}, "")

-- spider
function parseArch(archStr)
    if vmrContains(archStr, "x86-64") then
        return "amd64"
    end
    if vmrContains(archStr, "ARM64") then
        return "arm64"
    end
    return ""
end

function parseOs(osStr)
    if osStr == "macOS" then
        return "darwin"
    elseif osStr == "OS X" then
        return "darwin"
    elseif osStr == "Windows" then
        return "windows"
    elseif osStr == "Linux" then
        return "linux"
    end
    return ""
end

--[[
item{
    arch
    os
    url
    installer
    extra
    sum
    sum_type
    size
    lts
}
--]]
-- called by vmr
function crawl()
    local url = "https://golang.google.cn/dl/"
    local timeout = 600
    local headers = {}
    headers["User-Agent"] = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"
    local resp = vmrGetResponse(url, timeout, headers)

    local s1 = vmrInitSelection(resp, ".toggle")
    local s2 = vmrInitSelection(resp, ".toggleVisible")

    local versionList = vmrNewVersionList()

    function parseToggle(i, ss)
        if not ss then
            return
        end
        local versionStr = vmrAttr(ss, "id")
        versionStr = vmrTrimSpace(versionStr)

        if not vmrHasPrefix(versionStr, "go") then
           return
        end

        versionStr = vmrTrimPrefix(versionStr, "go")

        local downloadTable = vmrFind(ss, "table.downloadtable")
        local tr = vmrFind(downloadTable, "tr")

        function parseItem(i, sss)
            local tds = vmrFind(sss, "td")

            local eqs = vmrEq(tds, 1)
            local pkgKind = vmrTrimSpace(vmrText(eqs))

            eqs = vmrEq(tds, 3)
            local archInfo = parseArch(vmrText(eqs))

            eqs = vmrEq(tds, 2)
            local osInfo = parseOs(vmrText(eqs))

            if pkgKind == "Archive" and archInfo ~= "" and osInfo ~= "" then
                eqs = vmrEq(tds, 0)
                local a = vmrFind(eqs, "a")
                local href = vmrAttr(a, "href")
                if href == "" then
                    return
                end
                local item = {}
                item["arch"] = archInfo
                item["os"] = osInfo
                item["url"] = vmrUrlJoin("https://go.dev", href)
                item["installer"] = "unarchiver"

                eqs = vmrEq(tds, 4)
                item["extra"] = vmrTrimSpace(vmrText(eqs))

                eqs = vmrEq(tds, 5)
                item["sum"] = vmrTrimSpace(vmrText(eqs))

                if vmrLenString(item["sum"]) == 64 then
                    item["sum_type"] = "sha256"
                elseif vmrLenString(item["sum"]) == 40 then
                    item["sum_type"] = "sha1"
                end

                item["size"] = 0
                item["lts"] = ""

                vmrAddItem(versionList, versionStr, item)
            end
        end

        vmrEach(tr, parseItem)
    end

    vmrEach(s1, parseToggle)
    vmrEach(s2, parseToggle)

    return versionList
end`

func TestPlugin(t *testing.T) {
	p, err := NewPlugin("", pluginForGo)
	if err != nil {
		t.Error(err)
		return
	}
	defer p.Close()
	if err := p.Load(); err != nil {
		t.Error(err)
		return
	}

	vl, err := p.GetSDKVersions()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("find %d version items.\n", len(vl))
	}

	sdkName, err := p.GetSDKName()
	if err != nil {
		t.Error(err)
		return
	} else {
		assert.Equal(t, "go", sdkName, "should be 'go'!")
	}

	latestVersion, item := p.GetLatestVersion()
	if latestVersion == "" {
		t.Error("latest version is empty")
	} else {
		fmt.Printf("latest version: %s, %+v\n", latestVersion, item)
	}

	ic, err := p.GetInstallerConfig()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("installer config: %+v\n", ic)
		fmt.Printf("FlagFiles: %+v\n", ic.FlagFiles)
		fmt.Printf("BinaryDirs: %+v\n", ic.BinaryDirs)
		fmt.Printf("BinaryRename: %+v\n", ic.BinaryRename)
	}

	verItem := p.GetVersion("1.24.3")
	if verItem.Url == "" {
		t.Error("version item is nil")
	} else {
		fmt.Printf("version item: %+v\n", verItem)
	}

	sortedVersions := p.GetSortedVersions()
	if len(sortedVersions) == 0 {
		t.Error("sorted versions is empty")
	} else {
		fmt.Printf("sorted versions: %+v\n", sortedVersions)
	}
}

var pluginForLua = `--[[
    Lua from Conda.
--]]

-- global variables
sdk_name = "lua"
plugin_name = "lua"
plugin_version = "0.1"
prequisite = "conda"
homepage = "https://www.lua.org/"

-- installer config
ic = vmrNewInstallerConfig()
ic = vmrAddBinaryDirs(ic, "windows", {"Library", "bin"})
ic = vmrAddBinaryDirs(ic, "linux", {"bin"})
ic = vmrAddBinaryDirs(ic, "darwin", {"bin"})

-- spider
function crawl()
    local vl = vmrNewVersionList()
    local result = vmrSearchByConda(vl, "lua")
    return result
end`

func IsCondaInstalled() bool {
	homeDir, _ := os.UserHomeDir()
	_, err := gutils.ExecuteSysCommand(true, homeDir, "conda", "--help")
	return err == nil
}

func TestPluginLua(t *testing.T) {
	if ok := IsCondaInstalled(); !ok {
		t.Skip("conda is not installed")
	}

	p, err := NewPlugin("", pluginForLua)
	if err != nil {
		t.Error(err)
		return
	}
	defer p.Close()
	if err := p.Load(); err != nil {
		t.Error(err)
		return
	}

	vl, err := p.GetSDKVersions()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("find %d version items.\n", len(vl))
	}

	sdkName, err := p.GetSDKName()
	if err != nil {
		t.Error(err)
		return
	} else {
		assert.Equal(t, "lua", sdkName, "should be 'lua'!")
	}

	latestVersion, item := p.GetLatestVersion()
	if latestVersion == "" {
		t.Error("latest version is empty")
	} else {
		fmt.Printf("latest version: %s, %+v\n", latestVersion, item)
	}

	ic, err := p.GetInstallerConfig()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("installer config: %+v\n", ic)
		fmt.Printf("FlagFiles: %+v\n", ic.FlagFiles)
		fmt.Printf("BinaryDirs: %+v\n", ic.BinaryDirs)
		fmt.Printf("BinaryRename: %+v\n", ic.BinaryRename)
	}

	sortedVersions := p.GetSortedVersions()
	if len(sortedVersions) == 0 {
		t.Error("sorted versions is empty")
	} else {
		fmt.Printf("sorted versions: %+v\n", sortedVersions)
	}
}

var pluginForCoursier = `--[[
    Coursier.
    https://github.com/coursier/coursier
    https://github.com/VirtusLab/coursier-m1
--]]

-- global variables
sdk_name = "coursier"
plugin_name = "coursier"
plugin_version = "0.1"
prequisite = ""
homepage = "https://get-coursier.io/docs/cli-overview"

-- installer config
ic = vmrNewInstallerConfig()
ic = vmrAddFlagFiles(ic, "windows", {"cs.exe"})
ic = vmrAddFlagFiles(ic, "linux", {"cs"})
ic = vmrAddFlagFiles(ic, "darwin", {"cs"})
ic = vmrEnableFlagDirExcepted(ic)

--spider
local rePattern = "v(\\d+)(.\\d+){2}"
function tagFilter(str)
    local s = vmrRegexpFindString(rePattern, str)
	if s ~= "" then
		return true
	end
	return false
end

function versionParser(str)
	local s = vmrRegexpFindString(rePattern, str)
	s = vmrTrimPrefix(s, "v")
	return s
end

function fileFilter(str)
	if vmrHasPrefix(str, "cs-") and vmrHasSuffix(str, "-sdk.zip") then
        return true
    end
	return false
end

function archParser(str)
	if vmrContains(str, "-x86_64") then
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
	if vmrContains(str, "-win32") then
		return "windows"
	end
	return ""
end

function installerGetter(str)
	return "unarchiver"
end

-- called by vmr
function crawl()
    local r1 = vmrGetGithubRelease("coursier/coursier", tagFilter, versionParser, fileFilter, archParser, osParser, installerGetter)
    local r2 = vmrGetGithubRelease("VirtusLab/coursier-m1", tagFilter, versionParser, fileFilter, archParser, osParser, installerGetter)
    local result = vmrMergeVersionList(r1, r2)
    return result
end`

func TestPluginCoursier(t *testing.T) {
	p, err := NewPlugin("", pluginForCoursier)
	if err != nil {
		t.Error(err)
		return
	}
	defer p.Close()
	if err := p.Load(); err != nil {
		t.Error(err)
		return
	}

	vl, err := p.GetSDKVersions()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("find %d version items.\n", len(vl))
	}

	sdkName, err := p.GetSDKName()
	if err != nil {
		t.Error(err)
		return
	} else {
		assert.Equal(t, "coursier", sdkName, "should be 'coursier'!")
	}

	latestVersion, item := p.GetLatestVersion()
	if latestVersion == "" {
		t.Error("latest version is empty")
	} else {
		fmt.Printf("latest version: %s, %+v\n", latestVersion, item)
	}

	ic, err := p.GetInstallerConfig()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("installer config: %+v\n", ic)
		fmt.Printf("FlagFiles: %+v\n", ic.FlagFiles)
		fmt.Printf("BinaryDirs: %+v\n", ic.BinaryDirs)
		fmt.Printf("BinaryRename: %+v\n", ic.BinaryRename)
	}

	sortedVersions := p.GetSortedVersions()
	if len(sortedVersions) == 0 {
		t.Error("sorted versions is empty")
	} else {
		fmt.Printf("sorted versions: %+v\n", sortedVersions)
	}
}

var pluginForFlutter = `--[[
    Flutter.
    https://storage.googleapis.com/flutter_infra_release/releases/releases_linux.json
    https://storage.googleapis.com/flutter_infra_release/releases/releases_windows.json
    https://storage.googleapis.com/flutter_infra_release/releases/releases_macos.json
--]]

-- global variables
sdk_name = "flutter"
plugin_name = "flutter"
plugin_version = "0.1"
prequisite = ""
homepage = "https://flutter.dev/"

-- installer config
ic = vmrNewInstallerConfig()
ic = vmrAddFlagFiles(ic, "windows", {"README.md", "LICENSE", "CODEOWNERS"})
ic = vmrAddFlagFiles(ic, "linux", {"README.md", "LICENSE", "CODEOWNERS"})
ic = vmrAddFlagFiles(ic, "darwin", {"README.md", "LICENSE", "CODEOWNERS"})
ic = vmrEnableFlagDirExcepted(ic)
ic = vmrAddBinaryDirs(ic, "", {"bin"})

os, arch = vmrGetOsArch()

function getUrl()
	if os == "windows" then
		return "https://storage.googleapis.com/flutter_infra_release/releases/releases_windows.json"
	elseif os == "linux" then
		return "https://storage.googleapis.com/flutter_infra_release/releases/releases_linux.json"
	elseif os == "darwin" then
		return "https://storage.googleapis.com/flutter_infra_release/releases/releases_macos.json"
	end
end

local headers = {}
headers["User-Agent"] = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"
local url = getUrl()
local resp = vmrGetResponse(url, 10, headers)
local jj = vmrInitGJson(resp)
vl = vmrNewVersionList()

baseUrl = vmrGetString(jj, "base_url")

function parseArch(s)
	if s == "x64" then
		return "amd64"
	elseif s == "arm64" then
		return "arm64"
	else
		return ""
	end
end

function parseReleasesSlice(idx, release)
	local items = vmrInitGJson(release)
	item = {}
	item["version"] = vmrGetString(items, "version")
	local uri = vmrGetString(items, "archive")
	item["url"] = vmrUrlJoin(baseUrl, uri)
	item["sha256"] = vmrGetString(items, "sha256")
	if item["sha256"] ~= "" then
		item["sum_type"] = "sha256"
	end

	item["os"] = os

	item["arch"] = parseArch(vmrGetString(items, "dart_sdk_arch"))
	if item["arch"] == "" then
		return
	end

	item["size"] = 0
	vl = vmrAddItem(vl, item.version, item)
end

function crawl()
	vmrSliceEach(jj, "releases", parseReleasesSlice)
	return vl
end
`

func TestPluginFlutter(t *testing.T) {
	p, err := NewPlugin("", pluginForFlutter)
	if err != nil {
		t.Error(err)
		return
	}
	defer p.Close()
	if err := p.Load(); err != nil {
		t.Error(err)
		return
	}

	vl, err := p.GetSDKVersions()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("find %d version items.\n", len(vl))
	}

	sdkName, err := p.GetSDKName()
	if err != nil {
		t.Error(err)
		return
	} else {
		assert.Equal(t, "flutter", sdkName, "should be 'flutter'!")
	}

	latestVersion, item := p.GetLatestVersion()
	if latestVersion == "" {
		t.Error("latest version is empty")
	} else {
		fmt.Printf("latest version: %s, %+v\n", latestVersion, item)
	}

	ic, err := p.GetInstallerConfig()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("installer config: %+v\n", ic)
		fmt.Printf("FlagFiles: %+v\n", ic.FlagFiles)
		fmt.Printf("BinaryDirs: %+v\n", ic.BinaryDirs)
		fmt.Printf("BinaryRename: %+v\n", ic.BinaryRename)
	}

	sortedVersions := p.GetSortedVersions()
	if len(sortedVersions) == 0 {
		t.Error("sorted versions is empty")
	} else {
		fmt.Printf("sorted versions: %+v\n", sortedVersions)
	}
}

var pluginForMiniconda = `--[[
    Miniconda installation
--]]

-- global variables
sdk_name = "miniconda"
plugin_name = "miniconda"
plugin_version = "0.1"
prequisite = ""
homepage = "https://www.anaconda.com/docs/getting-started/miniconda/main"

-- installer config
ic = vmrNewInstallerConfig()
ic = vmrAddBinaryDirs(ic, "windows", { "bin" })
ic = vmrAddBinaryDirs(ic, "windows", { "condabin" })
ic = vmrAddBinaryDirs(ic, "linux", { "bin" })
ic = vmrAddBinaryDirs(ic, "linux", { "condabin" })
ic = vmrAddBinaryDirs(ic, "darwin", { "bin" })
ic = vmrAddBinaryDirs(ic, "darwin", { "condabin" })

-- spider
function parseArch(archStr)
    aa = vmrToLower(archStr)
    if vmrContains(aa, "x86_64") then
        return "amd64"
    end
    if vmrContains(aa, "arm64") then
        return "arm64"
    end
    if vmrContains(aa, "aarch64") then
        return "arm64"
    end
    return ""
end

function parseOs(osStr)
    oo = vmrToLower(osStr)
    if vmrContains(oo, "macosx") then
        return "darwin"
    elseif vmrContains(oo, "windows") then
        return "windows"
    elseif vmrContains(oo, "linux") then
        return "linux"
    end
    return ""
end

function parseVersionFromName(verName)
	local result = vmrRegexpFindString("(\\d+\\.\\d+\\.\\d+-\\d+)", verName)
	if result == "" then
	    return vmrRegexpFindString("(\\d+\\.\\d+\\.\\d+)", verName)
	end
	return result
end

function filterByHref(h)
    if vmrHasSuffix(h, ".pkg") then
        return false
    end
    return true
end

function crawl()
    local url = "https://repo.anaconda.com/miniconda/"
    local timeout = 600
    local headers = {}
    headers["User-Agent"] = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.36"
    local resp = vmrGetResponse(url, timeout, headers)
    -- vmrWriteFile("/home/moqsien/myprojects/go/version-manager/internal/luapi/plugin/miniconda.txt", resp)

    local os, arch = vmrGetOsArch()

    local tr = vmrInitSelection(resp, "tr")
    local versionList = vmrNewVersionList()

    baseUrl = "https://repo.anaconda.com/miniconda"

    function parseTr(i, ss)
        if not ss then
            return
        end

        local tds = vmrFind(ss, "td")
        local eqs = vmrEq(tds, 0)
        local a = vmrFind(eqs, "a")
        if not a then
	       	return
	    end

        local href = vmrAttr(a, "href")
        if not href or (not filterByHref(href)) then
            return
        end

        local itemOs = parseOs(href)
        local itemArch = parseArch(href)
        if itemOs ~= os or itemArch ~= arch then
            return
        end

        local versionName = parseVersionFromName(href)
        if versionName == "" then
        	print(href)
            return
        end

        local item = {}
        item["url"] = vmrUrlJoin(baseUrl, href)
        item["os"] = os
        item["arch"] = arch
        item["installer"] = "binary"

        eqs = vmrEq(tds, 3)
        item["sum"] = vmrTrimSpace(vmrText(eqs))
        if item["sum"] ~= "" then
            item["sum_type"] = "sha256"
        end
        item["size"] = 0

        vmrAddItem(versionList, versionName, item)
    end

    vmrEach(tr, parseTr)
    return versionList
end`

func TestPluginMiniconda(t *testing.T) {
	// return
	p, err := NewPlugin("", pluginForMiniconda)
	if err != nil {
		t.Error(err)
		return
	}
	defer p.Close()
	if err := p.Load(); err != nil {
		t.Error(err)
		return
	}

	vl, err := p.GetSDKVersions()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("find %d version items.\n", len(vl))
	}

	sdkName, err := p.GetSDKName()
	if err != nil {
		t.Error(err)
		return
	} else {
		assert.Equal(t, "miniconda", sdkName, "should be 'miniconda'!")
	}

	latestVersion, item := p.GetLatestVersion()
	if latestVersion == "" {
		t.Error("latest version is empty")
	} else {
		fmt.Printf("latest version: %s, %+v\n", latestVersion, item)
	}

	ic, err := p.GetInstallerConfig()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("installer config: %+v\n", ic)
		fmt.Printf("FlagFiles: %+v\n", ic.FlagFiles)
		fmt.Printf("BinaryDirs: %+v\n", ic.BinaryDirs)
		fmt.Printf("BinaryRename: %+v\n", ic.BinaryRename)
	}

	sortedVersions := p.GetSortedVersions()
	if len(sortedVersions) == 0 {
		t.Error("sorted versions is empty")
	} else {
		fmt.Printf("sorted versions: %+v\n", sortedVersions)
	}

	item = p.GetVersion("22.11.1-1")
	fmt.Println(item)
}

var pluginForRustup = `--[[
    Rustup installation
--]]

-- global variables
sdk_name = "rustup"
plugin_name = "rustup"
plugin_version = "0.1"
prequisite = ""
homepage = "https://rustup.rs/"

-- installer config
ic = vmrNewInstallerConfig()
ic = vmrAddFlagFiles(ic, "windows", { "rustup-init.exe" })
ic = vmrAddFlagFiles(ic, "linux", { "rustup-init" })
ic = vmrAddFlagFiles(ic, "darwin", { "rustup-init" })
ic = vmrEnableFlagDirExcepted(ic)

latestRustup = {
    "https://static.rust-lang.org/rustup/dist/x86_64-apple-darwin/rustup-init",
    "https://static.rust-lang.org/rustup/dist/aarch64-apple-darwin/rustup-init",
    "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init",
    "https://static.rust-lang.org/rustup/dist/aarch64-unknown-linux-gnu/rustup-init",
    "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe",
    "https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe",
}

function parseOs(u)
    if vmrContains(u, "windows") then
        return "windows"
    end
    if vmrContains(u, "linux") then
        return "linux"
    end
    if vmrContains(u, "darwin") then
        return "darwin"
    end
    return ""
end

function parseArch(u)
    if vmrContains(u, "x86_64") then
        return "amd64"
    end
    if vmrContains(u, "aarch64") then
        return "arm64"
    end
    return ""
end

os, arch = vmrGetOsArch()
-- spider
function crawl()
    local versionList = vmrNewVersionList()

    for i, url in ipairs(latestRustup) do
        local itemOs = parseOs(url)
        local itemArch = parseArch(url)
        if itemOs == os and itemArch == arch then
            local item = {}
            item["url"] = url
            item["os"] = os
            item["arch"] = arch
            item["installer"] = "executable"
            vmrAddItem(versionList, "latest", item)
        end
    end

    return versionList
end

-- TODO: post install handler
function postInstall(install_path)

end`

func TestPluginRustup(t *testing.T) {
	// return
	p, err := NewPlugin("", pluginForRustup)
	if err != nil {
		t.Error(err)
		return
	}
	defer p.Close()
	if err := p.Load(); err != nil {
		t.Error(err)
		return
	}

	vl, err := p.GetSDKVersions()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("find %d version items.\n", len(vl))
	}

	sdkName, err := p.GetSDKName()
	if err != nil {
		t.Error(err)
		return
	} else {
		assert.Equal(t, "rustup", sdkName, "should be 'rustup'!")
	}

	latestVersion, item := p.GetLatestVersion()
	if latestVersion == "" {
		t.Error("latest version is empty")
	} else {
		fmt.Printf("latest version: %s, %+v\n", latestVersion, item)
	}

	ic, err := p.GetInstallerConfig()
	if err != nil {
		t.Error(err)
		return
	} else {
		fmt.Printf("installer config: %+v\n", ic)
		fmt.Printf("FlagFiles: %+v\n", ic.FlagFiles)
		fmt.Printf("BinaryDirs: %+v\n", ic.BinaryDirs)
		fmt.Printf("BinaryRename: %+v\n", ic.BinaryRename)
	}

	sortedVersions := p.GetSortedVersions()
	if len(sortedVersions) == 0 {
		t.Error("sorted versions is empty")
	} else {
		fmt.Printf("sorted versions: %+v\n", sortedVersions)
	}
}
