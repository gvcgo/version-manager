$env:GOOS="darwin"
$env:GOARCH="arm64"
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/darwin-arm64/ ../cmd/vmr
Compress-Archive -Path ../build/darwin-arm64/vmr -DestinationPath ../build/vmr_darwin-arm64.zip

$env:GOOS="darwin"
$env:GOARCH="amd64"
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/darwin-amd64/ ../cmd/vmr
Compress-Archive -Path ../build/darwin-amd64/vmr -DestinationPath ../build/vmr_darwin-amd64.zip

$env:GOOS="linux"
$env:GOARCH="arm64"
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/linux-arm64/ ../cmd/vmr
Compress-Archive -Path ../build/linux-arm64/vmr -DestinationPath ../build/vmr_linux-arm64.zip

$env:GOOS="linux"
$env:GOARCH="amd64"
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/linux-amd64/ ../cmd/vmr
Compress-Archive -Path ../build/linux-amd64/vmr -DestinationPath ../build/vmr_linux-amd64.zip

$env:GOOS="windows"
$env:GOARCH="arm64"
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/windows-arm64/ ../cmd/vmr
Compress-Archive -Path ../build/windows-arm64/vmr.exe -DestinationPath ../build/vmr_windows-arm64.zip

$env:GOOS="windows"
$env:GOARCH="amd64"
go build -ldflags "-X main.GitTag=$(git describe --abbrev=0 --tags) -X main.GitHash=$(git show -s --format=%H)  -s -w" -o ../build/windows-amd64/ ../cmd/vmr
Compress-Archive -Path ../build/windows-amd64/vmr.exe -DestinationPath ../build/vmr_windows-amd64.zip
