# `.vmr.lock` session: `cd` outside the project ends the child shell. The destination is
# recorded so the outer cd hook can move the parent shell there once the session exits;
# otherwise the user would land back at the project root.
if ($env:VM_DISABLE -and $env:VMR_SESSION_PROJECT -and -not $env:VMR_SESSION_ACTIVE) {
    $env:VMR_SESSION_ACTIVE = '1'
    # A fresh pwsh has no previous location, so `cd -` would stay put; step through the
    # directory the user jumped from to seed it (no prompt hook is installed yet).
    $old = $env:VMR_SESSION_OLDPWD
    if ($old -and $old -ne $PWD.Path -and (Test-Path -LiteralPath $old)) {
        $origin = $PWD.Path
        Set-Location -LiteralPath $old
        Set-Location -LiteralPath $origin
    }
    function _vmr_session_cd_hook {
        $global:VMR_CDHOOK_LAST_PWD = $PWD.Path
        $project = $env:VMR_SESSION_PROJECT.TrimEnd('\', '/')
        $sep = [System.IO.Path]::DirectorySeparatorChar
        if ($PWD.Path -eq $project) { return }
        if ($PWD.Path.StartsWith($project + $sep, [System.StringComparison]::OrdinalIgnoreCase)) { return }
        Set-Content -LiteralPath __VMR_SESSION_EXIT__ -Value $PWD.Path
        exit
    }

    function cdhook {
        if ($args.Count -eq 0) { Set-Location } else { Set-Location @args }
        _vmr_session_cd_hook
    }

    Set-Alias -Name cd -Option AllScope -Value cdhook

    # `z`/`zi` run the module-qualified `Set-Location`, bypassing the alias; `prompt` is the
    # only hook PowerShell offers, so watch the path there to still end the session on exit.
    function global:_vmr_pwd_watch {
        if ($PWD.Path -eq $global:VMR_CDHOOK_LAST_PWD) { return }
        $global:VMR_CDHOOK_LAST_PWD = $PWD.Path
        _vmr_session_cd_hook
    }

    if (-not $global:__vmr_session_prompt_installed) {
        $global:__vmr_session_prompt_installed = $true
        $global:__vmr_session_prompt = $function:prompt
        function global:prompt {
            _vmr_pwd_watch
            if ($global:__vmr_session_prompt) { & $global:__vmr_session_prompt }
            else { "PS $($executionContext.SessionState.Path.CurrentLocation)> " }
        }
    }
}
