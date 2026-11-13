# cd hook start
export PATH="__VMR_INSTALL__:${PATH}"

if [ -z "$VM_DISABLE" ]; then
    cdhook() {
        if [ $# -eq 0 ]; then
            builtin cd
        else
            builtin cd "$@" && _vmr_cd_hook
        fi
    }
    alias cd='cdhook'

    # `vmr cdhook` walks up from the working directory for `.vmr.lock` and loads the Lua
    # plugins on every directory change. It records the applied lock directory in the file
    # below; this hook keeps that value in `VMR_LOCKFILE_DIR` and skips the call while the
    # shell stays inside the same project. Leaving the project, or entering a directory that
    # has its own `.vmr.lock`, looks it up again.
    _vmr_cd_hook() {
        VMR_CDHOOK_LAST_PWD=${PWD}
        local _lockdir="${VMR_LOCKFILE_DIR-}"
        if [ -n "${_lockdir}" ] && [ -f "${_lockdir}/.vmr.lock" ]; then
            case "${PWD}" in
                "${_lockdir}") return 0 ;;
            esac
            case "${PWD}/" in
                "${_lockdir%/}/"*)
                    [ -f "${PWD}/.vmr.lock" ] || return 0
                    ;;
            esac
        fi
        # `cd -` in the session shell needs the directory the user jumped from.
        VMR_SESSION_OLDPWD="${OLDPWD-}" vmr cdhook
        local _status=$?
        VMR_LOCKFILE_DIR=
        if [ -f "__VMR_LOCKFILE_DIR__" ]; then
            IFS= read -r VMR_LOCKFILE_DIR < "__VMR_LOCKFILE_DIR__"
        fi
        export VMR_LOCKFILE_DIR
        # A `.vmr.lock` session that ended because `cd` left its project records the
        # destination; move this shell there before looking up the new directory.
        if [ -f "__VMR_SESSION_EXIT__" ]; then
            local _dir
            IFS= read -r _dir < "__VMR_SESSION_EXIT__"
            rm -f "__VMR_SESSION_EXIT__"
            if [ -n "${_dir}" ] && [ "${_dir}" != "${PWD}" ] && builtin cd -- "${_dir}" 2>/dev/null; then
                _vmr_cd_hook
                return $?
            fi
        fi
        return "${_status}"
    }

    # `z`/`zi`, `pushd`, `popd` and script-driven jumps change PWD without going through the
    # `cd` alias, so watch the directory itself; the alias keeps `cd proj && cmd` immediate.
    _vmr_pwd_watch() {
        local _st=$?
        if [ "${PWD}" != "${VMR_CDHOOK_LAST_PWD-}" ]; then
            VMR_CDHOOK_LAST_PWD=${PWD}
            _vmr_cd_hook
        fi
        return "${_st}"
    }

    case " ${PROMPT_COMMAND[*]-} " in
        *" _vmr_pwd_watch "*) ;;
        *)
            case "$(declare -p PROMPT_COMMAND 2>/dev/null)" in
                "declare -a PROMPT_COMMAND="*) PROMPT_COMMAND=(_vmr_pwd_watch "${PROMPT_COMMAND[@]}") ;;
                *) PROMPT_COMMAND="_vmr_pwd_watch${PROMPT_COMMAND:+;${PROMPT_COMMAND}}" ;;
            esac
            ;;
    esac

    if [ -z "${VMR_CDHOOK_ACTIVE}" ]; then
        export VMR_CDHOOK_ACTIVE=1
        cdhook "$(pwd)"
    fi
fi
__VMR_SESSION_HOOK__
# cd hook end
