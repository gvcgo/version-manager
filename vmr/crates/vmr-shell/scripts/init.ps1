__VMR_SESSION_HOOK__
if (-not $env:VM_DISABLE) {
    $env:Path = (Join-Path $HOME '.vmr') + ';' + $env:Path
__VMR_HOOK__
__VMR_REFRESH__
}
function global:svmr { $env:VM_DISABLE = ''; . $PROFILE }
