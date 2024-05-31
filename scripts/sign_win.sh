#!/bin/bash

# Sign for windows executables.

mv ../build/windows-amd64/vmr.exe ../build/windows-amd64/vmr_old.exe
mv ../build/windows-arm64/vmr.exe ../build/windows-arm64/vmr_old.exe

osslsigncode sign -addUnauthenticatedBlob -pkcs12 ./vmr.pfx -pass Vmr2024 -n "GVC" -i https://github.com/gvcgo/ -in ../build/windows-amd64/vmr_old.exe -out ../build/windows-amd64/vmr.exe
rm ../build/windows-amd64/vmr_old.exe

osslsigncode sign -addUnauthenticatedBlob -pkcs12 ./vmr.pfx -pass Vmr2024 -n "GVC" -i https://github.com/gvcgo/ -in ../build/windows-arm64/vmr_old.exe -out ../build/windows-arm64/vmr.exe
rm ../build/windows-arm64/vmr_old.exe
