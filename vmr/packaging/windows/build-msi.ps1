# Build the vmr MSI with the portable WiX Toolset v3 binaries (no admin install required).
#
#   powershell -ExecutionPolicy Bypass -File packaging\windows\build-msi.ps1
#   powershell -ExecutionPolicy Bypass -File packaging\windows\build-msi.ps1 -SkipBuild
#
# Output: target\windows\vmr-<version>-x64.msi
param(
    [ValidateSet("release", "debug")]
    [string]$Configuration = "release",
    [string]$Version = "",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $Root

if (-not $Version) {
    $match = Select-String -LiteralPath (Join-Path $Root "Cargo.toml") `
        -Pattern '^\s*version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if (-not $match) { throw "could not read the workspace version from Cargo.toml" }
    $Version = $match.Matches[0].Groups[1].Value
}

if (-not $SkipBuild) {
    if ($Configuration -eq "release") {
        & cargo build --release -p vmr-cli --bin vmr
    } else {
        & cargo build -p vmr-cli --bin vmr
    }
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
}

$SourceDir = Join-Path $Root "target\$Configuration"
$SourceExe = Join-Path $SourceDir "vmr.exe"
if (-not (Test-Path $SourceExe)) { throw "missing binary: $SourceExe" }

$WixDir = Join-Path $Root "target\wix314"
$Zip = Join-Path $Root "target\wix314-binaries.zip"
$Candle = Get-ChildItem -LiteralPath $WixDir -Filter candle.exe -Recurse -ErrorAction SilentlyContinue |
    Select-Object -First 1
if (-not $Candle) {
    if (-not (Test-Path $Zip)) {
        $Upstream = "github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip"
        # GitHub release assets are flaky on some networks; try a proxy mirror first.
        $Urls = @("https://gh-proxy.com/https://$Upstream", "https://$Upstream")
        $Curl = Get-Command curl.exe -ErrorAction SilentlyContinue
        $Downloaded = $false
        foreach ($Url in $Urls) {
            Write-Host "Downloading WiX Toolset v3.14: $Url"
            if (Test-Path $Zip) { Remove-Item -LiteralPath $Zip -Force }
            if ($Curl) {
                & $Curl.Source -L --fail --retry 3 --retry-delay 2 -o $Zip $Url
                $Ok = ($LASTEXITCODE -eq 0)
            } else {
                try {
                    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Zip
                    $Ok = $true
                } catch {
                    $Ok = $false
                }
            }
            if ($Ok -and (Test-Path $Zip)) {
                $Downloaded = $true
                break
            }
        }
        if (-not $Downloaded) { throw "downloading WiX failed" }
    }
    New-Item -ItemType Directory -Force -Path $WixDir | Out-Null
    Expand-Archive -LiteralPath $Zip -DestinationPath $WixDir -Force
    $Candle = Get-ChildItem -LiteralPath $WixDir -Filter candle.exe -Recurse |
        Select-Object -First 1
    if (-not $Candle) { throw "candle.exe not found in the WiX archive" }
}
$WixBin = $Candle.DirectoryName
$Light = Join-Path $WixBin "light.exe"
if (-not (Test-Path $Light)) { throw "light.exe not found next to candle.exe" }

$OutDir = Join-Path $Root "target\windows"
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$WixObj = Join-Path $OutDir "vmr.wixobj"
$Msi = Join-Path $OutDir "vmr-$Version-x64.msi"

& $Candle.FullName -nologo -arch x64 "-dVersion=$Version" "-dSourceDir=$SourceDir" `
    -out $WixObj (Join-Path $PSScriptRoot "vmr.wxs")
if ($LASTEXITCODE -ne 0) { throw "candle failed" }

& $Light -nologo -ext WixUtilExtension -out $Msi $WixObj
if ($LASTEXITCODE -ne 0) { throw "light failed" }

Write-Host "MSI: $Msi"
