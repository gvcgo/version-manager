cdhook() {
    if [ $# -eq 0 ]; then
        builtin cd
    else
        builtin cd "$@" && _vmr_cd_hook
    fi
}
alias cd='cdhook'

# `vmr cdhook` walks up from the working directory for `.vmr.lock` and loads the Lua plugins
# on every directory change. It records the applied project in the cache file below; this
# hook keeps that value in `VMR_CD_CACHE` (a child process cannot export to this shell) and
# skips the call while the shell stays inside the same project. Leaving the project, or
# entering a directory that has its own `.vmr.lock`, looks it up again.
_vmr_cd_hook() {
    local _cache="${VMR_CD_CACHE-}"
    if [ -n "${_cache}" ] && [ -f "${_cache}/.vmr.lock" ]; then
        case "${PWD}" in
            "${_cache}") return 0 ;;
        esac
        case "${PWD}/" in
            "${_cache%/}/"*)
                [ -f "${PWD}/.vmr.lock" ] || return 0
                ;;
        esac
    fi
    vmr cdhook
    local _status=$?
    VMR_CD_CACHE=
    if [ -f "__VMR_CD_CACHE__" ]; then
        IFS= read -r VMR_CD_CACHE < "__VMR_CD_CACHE__"
    fi
    export VMR_CD_CACHE
    return "${_status}"
}

if [ -z "${VMR_CD_INIT}" ]; then
    VMR_CD_INIT="vmr_cd_init"
    cdhook "$(pwd)"
fi
