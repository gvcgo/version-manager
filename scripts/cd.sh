cdhook() {
    if [ -d "$1" ];then
        cd "$1"
        vmr use -E
    fi
}

alias cd='cdhook'