$sysType=systeminfo | find "System Type"

Write-Host "$sysType"

if ( "$sysType" -match "x64-based" )
{
    $arch="amd64"
}

$version="v0.1.1"

$filename="vm_windows-" + $arch + ".zip"
$download_url="https://gvc.1710717.xyz/proxy/https://github.com/gvcgo/version-manager/releases/download/"

$url=$download_url + $version + "/" + $filename

Write-Host "Downloading files..."

Invoke-RestMethod -Uri $url -OutFile $filename

$TRUE_FALSE=(Test-Path $filename)
if ( $TRUE_FALSE -eq "True" )
{
   Expand-Archive -Path $filename -DestinationPath .\
    $TRUE_FALSE=(Test-Path "vm.exe")
    if ( $TRUE_FALSE -eq "True" )
    {
        .\vm.exe i
        remove-Item -Recurse -Force .\vm.exe
    }
    remove-Item -Recurse -Force $filename
}