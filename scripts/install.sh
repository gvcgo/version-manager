#!/bin/sh
say() {
    printf 'rustup: %s\n' "$1"
}

err() {
    say "$1" >&2
    exit 1
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1' (command not found)"
    fi
}

main() {
    local os_type="$(uname -s)"
    local os_arch="$(uname -m)"
    local version="v0.1.1"
    local download_url="https://gvc.1710717.xyz/proxy/https://github.com/gvcgo/version-manager/releases/download/"
    
    local osType="linux"
    if [ "$os_type" = "Darwin" ]; then
        osType="darwin"
    fi

    local osArch="amd64"

    if  [ "$os_arch" = "arm64" ] && [ "$os_type" = "aarch64" ]; then
        osArch="arm64"
    fi

    local filename="vm_$osType-$osArch.zip"
    local url="$download_url$version/$filename"
    echo "$url"
    need_cmd "curl"
    need_cmd "unzip"
    need_cmd "mkdir"

    curl -o "$filename" "$url"

    if [ -s "./$filename" ]; then
        unzip "./$filename"
        chmod +x "./vm"
        if [ -s "./vm" ]; then
            ./vm i
        fi
        rm -rf "./$filename"
        rm -rf "./vm"
    fi
}

main
