function cdhook {
    if (Test-Path $args[0]) {
        chdir $args[0]
        vmr cdhook
    }
}

Set-Alias -Name cd -Option AllScope -Value cdhook

if ("" -eq "$env:VMR_CD_INIT") {
    $env:VMR_CD_INIT = "vmr_cd_init"
    cdhook (Get-Location).Path
}
