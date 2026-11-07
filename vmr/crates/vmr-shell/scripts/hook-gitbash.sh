# cd hook start
export PATH="__VMR_INSTALL__:${PATH}"

if [ -z "$VM_DISABLE" ]; then
    cdhook() {
        if [ $# -eq 0 ]; then
            builtin cd
        else
            builtin cd "$@" && vmr cdhook
        fi
    }
    alias cd='cdhook'

    if [ -z "${VMR_CD_INIT}" ]; then
        VMR_CD_INIT="vmr_cd_init"
        cdhook "$(pwd)"
    fi
fi
# cd hook end