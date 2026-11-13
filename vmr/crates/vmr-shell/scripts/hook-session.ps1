# `.vmr.lock` session: `cd` outside the project ends the child shell. The destination is
# recorded so the outer cd hook can move the parent shell there once the session exits;
# otherwise the user would land back at the project root.
if ($env:VM_DISABLE -and $env:VMR_SESSION_PROJECT -and -not $env:VMR_SESSION_ACTIVE) {
    $env:VMR_SESSION_ACTIVE = '1'
    function _vmr_session_cd_hook {
        $project = $env:VMR_SESSION_PROJECT.TrimEnd('\', '/')
        $sep = [System.IO.Path]::DirectorySeparatorChar
        if ($PWD.Path -eq $project) { return }
        if ($PWD.Path.StartsWith($project + $sep, [System.StringComparison]::OrdinalIgnoreCase)) { return }
        Set-Content -LiteralPath __VMR_SESSION_EXIT__ -Value $PWD.Path
        exit
    }

    function cdhook {
        if (Test-Path $args[0]) {
            chdir $args[0]
            _vmr_session_cd_hook
        }
    }

    Set-Alias -Name cd -Option AllScope -Value cdhook
}
