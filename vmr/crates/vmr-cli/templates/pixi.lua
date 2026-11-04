-- Plugin template: pixi / conda-forge repodata.
-- Copy to ~/.vmr/plugins/<sdk>.lua (or the vmr_plugins repo) and edit the placeholders.
local C = dofile(__vmr_plugin_dir .. "/_vmr_common.lua")

plugin_name = "example"
sdk_name = "example"
plugin_version = "0.1"
homepage = "https://example.com/"

ic = C.installer({
    flag_files = {
        windows = { "Library", "Scripts" },
        darwin = { "bin" },
        linux = { "bin" },
    },
    binary_dirs = {
        windows = { { "Library", "bin" }, { "Scripts" } },
        darwin = { { "bin" } },
        linux = { { "bin" } },
    },
})

function crawl()
    -- `C.pixi` searches the conda-forge repodata cache/channel for the package name.
    -- Combine with `C.github` via `vmrMergeVersionList` when upstream also ships archives.
    return C.pixi("example")
end
