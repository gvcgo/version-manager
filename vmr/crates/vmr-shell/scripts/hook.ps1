# `vmr cdhook` walks up from the working directory for `.vmr.lock` and loads the Lua plugins
# on every directory change. It records the applied lock directory in the file below; this
# hook keeps that value in `VMR_LOCKFILE_DIR` and skips the call while the shell stays inside
# the same project. Leaving the project, or entering a directory that has its own
# `.vmr.lock`, looks it up again.
function _vmr_cd_hook {
    $global:VMR_CDHOOK_LAST_PWD = $PWD.Path
    $lockdir = $env:VMR_LOCKFILE_DIR
    if ($lockdir -and (Test-Path -LiteralPath (Join-Path $lockdir '.vmr.lock'))) {
        if ($PWD.Path -eq $lockdir) { return }
        $sep = [System.IO.Path]::DirectorySeparatorChar
        $prefix = $lockdir.TrimEnd('\', '/') + $sep
        if ($PWD.Path.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
            if (-not (Test-Path -LiteralPath (Join-Path $PWD.Path '.vmr.lock'))) { return }
        }
    }
    vmr cdhook
    $raw = Get-Content -Raw -LiteralPath __VMR_LOCKFILE_DIR__ -ErrorAction SilentlyContinue
    if ($null -eq $raw) { $env:VMR_LOCKFILE_DIR = '' } else { $env:VMR_LOCKFILE_DIR = $raw.Trim() }
    # A `.vmr.lock` session that ended because `cd` left its project records the destination;
    # move this shell there before looking up the new directory (it may have its own lock).
    if (Test-Path -LiteralPath __VMR_SESSION_EXIT__) {
        $dir = Get-Content -Raw -LiteralPath __VMR_SESSION_EXIT__ -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath __VMR_SESSION_EXIT__ -Force -ErrorAction SilentlyContinue
        if ($dir) { $dir = $dir.Trim() }
        if ($dir -and $dir -ne $PWD.Path -and (Test-Path -LiteralPath $dir)) {
            chdir $dir
            _vmr_cd_hook
            return
        }
    }
}

function cdhook {
    # Delegate to `Set-Location` so `cd -`, `cd ~` and provider paths keep working; a bad
    # path throws exactly as it would without the alias, and the hook only runs on success.
    $env:VMR_SESSION_OLDPWD = $global:VMR_CDHOOK_LAST_PWD
    if ($args.Count -eq 0) { Set-Location } else { Set-Location @args }
    _vmr_cd_hook
}

Set-Alias -Name cd -Option AllScope -Value cdhook

# `z`/`zi` run `Microsoft.PowerShell.Management\Set-Location`, which bypasses the `cd` alias
# (module-qualified cmdlets skip aliases and functions), and PowerShell has no directory-change
# event: the host calls `prompt` before every prompt, so watch the path there as well. The
# alias stays for the same-line case (`cd proj; cmd`), which `prompt` only sees after the line.
function global:_vmr_pwd_watch {
    if ($PWD.Path -eq $global:VMR_CDHOOK_LAST_PWD) { return }
    $env:VMR_SESSION_OLDPWD = $global:VMR_CDHOOK_LAST_PWD
    $global:VMR_CDHOOK_LAST_PWD = $PWD.Path
    _vmr_cd_hook
}

if (-not $global:__vmr_prompt_installed) {
    $global:__vmr_prompt_installed = $true
    $global:__vmr_prompt = $function:prompt
    function global:prompt {
        _vmr_pwd_watch
        if ($global:__vmr_prompt) { & $global:__vmr_prompt }
        else { "PS $($executionContext.SessionState.Path.CurrentLocation)> " }
    }
}

if ("" -eq "$env:VMR_CDHOOK_ACTIVE") {
    $env:VMR_CDHOOK_ACTIVE = 1
    cdhook (Get-Location).Path
}
