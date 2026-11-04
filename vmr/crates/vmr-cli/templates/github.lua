-- Plugin template: GitHub Releases.
-- Copy to ~/.vmr/plugins/<sdk>.lua (or the vmr_plugins repo) and edit the placeholders.
local C = dofile(__vmr_plugin_dir .. "/_vmr_common.lua")

plugin_name = "example"
sdk_name = "example"
plugin_version = "0.1"
homepage = "https://github.com/owner/repo"

ic = C.installer({
    flag_files = {
        windows = { "bin" },
        darwin = { "bin" },
        linux = { "bin" },
    },
    binary_dirs = {
        windows = { { "bin" } },
        darwin = { { "bin" } },
        linux = { { "bin" } },
    },
    flag_dir_excepted = true,
})

function crawl()
    return C.github({
        repo = "owner/repo",
        tag_filter = function(tag)
            return vmrRegexpFindString("^v?[0-9]+\\.[0-9]+\\.[0-9]+$", tag) ~= ""
        end,
        version_parser = function(tag)
            return vmrTrimPrefix(tag, "v")
        end,
        file_filter = function(name)
            return vmrHasSuffix(name, ".tar.gz") or vmrHasSuffix(name, ".zip")
        end,
        os_parser = function(name)
            if vmrContains(name, "windows") then
                return "windows"
            end
            if vmrContains(name, "darwin") or vmrContains(name, "macos") then
                return "darwin"
            end
            return "linux"
        end,
        arch_parser = function(name)
            if vmrContains(name, "aarch64") or vmrContains(name, "arm64") then
                return "arm64"
            end
            if vmrContains(name, "x86_64") or vmrContains(name, "amd64") then
                return "amd64"
            end
            return "any"
        end,
        installer = vmrInstallerUnarchiver,
    })
end
