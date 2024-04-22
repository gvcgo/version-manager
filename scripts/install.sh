#!/bin/sh
# ====================================================
# Installs/Updates version-manager(vm) for MacOS/Linux
# ====================================================
# Copyright (c) 2024 moqsien@hotmail.com
#
# Permission is hereby granted, free of charge, to any person obtaining a copy of
# this software and associated documentation files (the "Software"), to deal in
# the Software without restriction, including without limitation the rights to
# use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
# the Software, and to permit persons to whom the Software is furnished to do so,
# subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
# FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
# COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
# IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
# CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

say() {
    printf 'vm: %s\n' "$1"
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

    local version="v0.1.7"
    
    local download_url="https://gvc.1710717.xyz/proxy/https://github.com/gvcgo/version-manager/releases/download/"
    local osType="linux"
    if [ "$os_type" = "Darwin" ]; then
        osType="darwin"
    fi

    local osArch="amd64"

    if  [ "$os_arch" = "arm64" ] || [ "$os_type" = "aarch64" ]; then
        osArch="arm64"
    fi

    local filename="vmr_$osType-$osArch.zip"
    local url="$download_url$version/$filename"
    echo "$url"
    need_cmd "curl"
    need_cmd "unzip"
    need_cmd "mkdir"

    echo "Downloading files..."

    curl -o "$filename" "$url"

    echo "Installing..."

    if [ -s "./$filename" ]; then
        unzip "./$filename"
        chmod +x "./vmr"
        if [ -s "./vmr" ]; then
            ./vmr i
        fi
        rm -rf "./$filename"
        rm -rf "./vmr"
    fi
}

main
