<#
 @@    Copyright (c) 2024 moqsien@hotmail.com
 @@
 @@    Permission is hereby granted, free of charge, to any person obtaining a copy of
 @@    this software and associated documentation files (the "Software"), to deal in
 @@    the Software without restriction, including without limitation the rights to
 @@    use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 @@    the Software, and to permit persons to whom the Software is furnished to do so,
 @@    subject to the following conditions:
 @@
 @@    The above copyright notice and this permission notice shall be included in all
 @@    copies or substantial portions of the Software.
 @@
 @@    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 @@    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 @@    FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 @@    COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 @@    IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 @@    CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 #>

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