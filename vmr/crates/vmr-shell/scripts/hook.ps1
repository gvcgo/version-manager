# `vmr cdhook` walks up from the working directory for `.vmr.lock` and loads the Lua plugins
# on every directory change. It records the applied project in the cache file below; this
# hook keeps that value in `VMR_CD_CACHE` and skips the call while the shell stays inside the
# same project. Leaving the project, or entering a directory that has its own `.vmr.lock`,
# looks it up again.
function _vmr_cd_hook {
    $cache = $env:VMR_CD_CACHE
    if ($cache -and (Test-Path -LiteralPath (Join-Path $cache '.vmr.lock'))) {
        if ($PWD.Path -eq $cache) { return }
        $sep = [System.IO.Path]::DirectorySeparatorChar
        $prefix = $cache.TrimEnd('\', '/') + $sep
        if ($PWD.Path.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            if (-not (Test-Path -LiteralPath (Join-Path $PWD.Path '.vmr.lock'))) { return }
        }
    }
    vmr cdhook
    $raw = Get-Content -Raw -LiteralPath __VMR_CD_CACHE__ -ErrorAction SilentlyContinue
    if ($null -eq $raw) { $env:VMR_CD_CACHE = '' } else { $env:VMR_CD_CACHE = $raw.Trim() }
}

function cdhook {
    if (Test-Path $args[0]) {
        chdir $args[0]
        _vmr_cd_hook
    }
}

Set-Alias -Name cd -Option AllScope -Value cdhook

if ("" -eq "$env:VMR_CD_INIT") {
    $env:VMR_CD_INIT = "vmr_cd_init"
    cdhook (Get-Location).Path
}
