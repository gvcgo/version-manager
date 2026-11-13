# `.vmr.lock` session: `cd` outside the project ends the child shell. The destination is
# recorded so the outer cd hook can move the parent shell there once the session exits;
# otherwise the user would land back at the project root.
if [ -n "${VM_DISABLE-}" ] && [ -n "${VMR_SESSION_PROJECT-}" ] \
    && [ -z "${VMR_SESSION_ACTIVE-}" ]; then
    export VMR_SESSION_ACTIVE=1
    _vmr_session_cd_hook() {
        local _pwd _project
        _pwd="$(pwd -P)"
        _project="${VMR_SESSION_PROJECT}"
        # Compare in POSIX form: git-bash receives native Windows paths.
        if command -v cygpath >/dev/null 2>&1; then
            case "${_pwd}" in
                /*) ;;
                *) _pwd="$(cygpath -u "${_pwd}")" ;;
            esac
            case "${_project}" in
                /*) ;;
                *) _project="$(cygpath -u "${_project}")" ;;
            esac
        fi
        case "${_pwd}" in
            "${_project}" | "${_project%/}"/*) return 0 ;;
        esac
        printf '%s\n' "${PWD}" > "__VMR_SESSION_EXIT__"
        exit
    }

    cdhook() {
        if [ $# -eq 0 ]; then
            builtin cd
        else
            builtin cd "$@" || return
        fi
        _vmr_session_cd_hook
    }
    alias cd='cdhook'
fi
