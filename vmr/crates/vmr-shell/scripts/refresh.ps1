function global:vmrsource {
    $versions = [System.Environment]::GetEnvironmentVariable('VMR_VERSIONS', 'User')
    if (-not $versions) { $versions = __VMR_VERSIONS_DIR__ }
    if ($versions) { $env:VMR_VERSIONS = $versions }
    $isVmPath = {
        param($entry)
        if (-not $entry) { return $false }
        $p = $entry.Replace('/', '\').TrimEnd('\')
        if ($p.StartsWith('%VMR_VERSIONS%', [System.StringComparison]::OrdinalIgnoreCase)) { return $true }
        if (-not $versions) { return $false }
        $v = $versions.Replace('/', '\').TrimEnd('\')
        return ($p -ieq $v) -or $p.StartsWith($v + '\', [System.StringComparison]::OrdinalIgnoreCase)
    }
    $userPath = [System.Environment]::GetEnvironmentVariable('Path', 'User')
    $vmrEntries = @()
    if ($userPath) {
        foreach ($entry in $userPath.Split(';')) {
            if (-not (& $isVmPath $entry)) { continue }
            $expanded = $entry
            if ($expanded.StartsWith('%VMR_VERSIONS%') -and $versions) {
                $expanded = $versions + $expanded.Substring(('%VMR_VERSIONS%').Length)
            }
            if (-not ($vmrEntries -contains $expanded)) { $vmrEntries += $expanded }
        }
    }
    $current = $env:Path
    if (-not $current) { $current = '' }
    $kept = @()
    foreach ($entry in $current.Split(';')) {
        if ($entry -and -not (& $isVmPath $entry)) { $kept += $entry }
    }
    $env:Path = (@($vmrEntries) + $kept) -join ';'
    $keyFile = __VMR_ENV_KEYS__
    if (Test-Path -LiteralPath $keyFile) {
        foreach ($key in @(Get-Content -LiteralPath $keyFile)) {
            $key = $key.Trim()
            if (-not $key -or $key -ieq 'Path') { continue }
            $value = [System.Environment]::GetEnvironmentVariable($key, 'User')
            if ($null -ne $value) {
                Set-Item -Path ('Env:' + $key) -Value $value
            } else {
                Remove-Item -Path ('Env:' + $key) -ErrorAction SilentlyContinue
            }
        }
    }
}
$global:__vmr_exe = $null
$vmrCommand = Get-Command -Name 'vmr' -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($vmrCommand) { $global:__vmr_exe = $vmrCommand.Source }
if (-not $global:__vmr_exe) {
    $vmrFallback = Join-Path $HOME '.vmr\vmr.exe'
    if (Test-Path -LiteralPath $vmrFallback) { $global:__vmr_exe = $vmrFallback }
}
if ($global:__vmr_exe) {
    function global:vmr {
        & $global:__vmr_exe @args
        $vmrExit = $LASTEXITCODE
        if ($vmrExit -eq 0 -and $args.Count -gt 0 -and (@('use', 'u', 'h', 'uninstall', 'uni', 'r') -contains [string]$args[0])) {
            vmrsource
        }
        $global:LASTEXITCODE = $vmrExit
    }
}
