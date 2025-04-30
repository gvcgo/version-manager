#!/bin/zsh

export GOOS="darwin"
export GOARCH="arm64"
rm -rf ../build/vmr_darwin-arm64.zip
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/darwin-arm64/ ../cmd/vmr
zip -j  ../build/vmr_darwin-arm64.zip ../build/darwin-arm64/vmr 

export GOOS="darwin"
export GOARCH="amd64"
rm -rf ../build/vmr_darwin-amd64.zip
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/darwin-amd64/ ../cmd/vmr
zip -j  ../build/vmr_darwin-amd64.zip ../build/darwin-amd64/vmr

export GOOS="linux"
export GOARCH="arm64"
rm -rf ../build/vmr_linux-arm64.zip
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/linux-arm64/ ../cmd/vmr
zip -j ../build/vmr_linux-arm64.zip ../build/linux-arm64/vmr  

export GOOS="linux"
export GOARCH="amd64"
rm -rf ../build/vmr_linux-amd64.zip
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/linux-amd64/ ../cmd/vmr
zip -j ../build/vmr_linux-amd64.zip ../build/linux-amd64/vmr 

export GOOS="windows"
export GOARCH="arm64"
rm -rf ../build/vmr_windows-arm64.zip
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/windows-arm64/ ../cmd/vmr
zip -j ../build/vmr_windows-arm64.zip ../build/windows-arm64/vmr.exe 

export GOOS="windows"
export GOARCH="amd64"
rm -rf ../build/vmr_windows-amd64.zip
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/windows-amd64/ ../cmd/vmr
zip -j ../build/vmr_windows-amd64.zip ../build/windows-amd64/vmr.exe 
