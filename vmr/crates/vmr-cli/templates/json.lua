-- Plugin template: JSON API crawl.
-- Copy to ~/.vmr/plugins/<sdk>.lua (or the vmr_plugins repo) and edit the placeholders.
local C = dofile(__vmr_plugin_dir .. "/_vmr_common.lua")

plugin_name = "example"
sdk_name = "example"
plugin_version = "0.1"
homepage = "https://example.com/"

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
})

local VERSIONS_URL = "https://example.com/versions.json"

function crawl()
    local vl = vmrNewVersionList()
    local resp = vmrGetResponse(VERSIONS_URL, 300, {})
    local root = vmrInitGJson(resp)

    -- Edit the path/fields for the upstream JSON shape.
    vmrMapEach(root, "", function(version, entry)
        local url = vmrGetString(entry, "url")
        if url == "" then
            return
        end
        local os_name = vmrGetString(entry, "os")
        local arch = vmrGetString(entry, "arch")
        vmrAddItem(vl, vmrTrimPrefix(version, "v"), {
            url = url,
            arch = arch ~= "" and arch or "any",
            os = os_name ~= "" and os_name or "any",
            sum = vmrGetString(entry, "sha256"),
            sum_type = "sha256",
            size = vmrGetInt(entry, "size"),
            installer = vmrInstallerUnarchiver,
            lts = "",
            extra = "",
        })
    end)

    return vl
end
