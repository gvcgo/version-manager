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

$sysType = systeminfo | find "System Type"

Write-Host "$sysType"

$arch = "amd64"
if ("$sysType" -match "arm")
{
    $arch = "arm64"
}

$filename = "vmr_windows-" + $arch + ".zip"

$localpath = "..\build\" + $filename
if (Test-Path $filename)
{
    Remove-Item -Recurse -Force $filename
}
Write-Host "Copying files..."
Copy-Item $localpath -destination .\


$TRUE_FALSE = (Test-Path $filename)
if ($TRUE_FALSE -eq "True")
{
    Expand-Archive -Path $filename -DestinationPath .\
    $TRUE_FALSE = (Test-Path "vmr.exe")
    if ($TRUE_FALSE -eq "True")
    {
        .\vmr.exe i
        remove-Item -Recurse -Force .\vmr.exe
    }
    remove-Item -Recurse -Force $filename
}