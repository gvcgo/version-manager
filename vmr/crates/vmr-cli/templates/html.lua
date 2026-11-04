-- Plugin template: HTML page crawl.
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

local HOST = "https://example.com"
local INDEX_URL = HOST .. "/downloads"

function crawl()
    local vl = vmrNewVersionList()
    local seen = {}
    local resp = vmrGetResponse(INDEX_URL, 180, {})

    -- Edit the CSS selector for the release links on the page.
    vmrEach(vmrInitSelection(resp, "a.release"), function(_, anchor)
        local text = vmrTrimSpace(vmrText(anchor))
        local href = vmrAttr(anchor, "href")
        local version = vmrTrimPrefix(text, "v")
        if version == "" or href == "" or seen[version] then
            return
        end
        seen[version] = true

        vmrAddItem(vl, version, {
            url = vmrUrlJoin(INDEX_URL, href),
            arch = "amd64",
            os = "linux",
            sum = "",
            sum_type = "",
            size = 0,
            installer = vmrInstallerUnarchiver,
            lts = "",
            extra = "",
        })
    end)

    return vl
end
