# `.vmr.lock` session: `cd` outside the project ends the child shell. The destination is
# recorded so the outer cd hook can move the parent shell there once the session exits;
# otherwise the user would land back at the project root.
if [ -n "${VM_DISABLE-}" ] && [ -n "${VMR_SESSION_PROJECT-}" ] \
    && [ -z "${VMR_SESSION_ACTIVE-}" ]; then
    export VMR_SESSION_ACTIVE=1
    # zsh keeps the previous directory internally and ignores assignments to OLDPWD, so seed
    # it by stepping through the directory the user jumped from; `cd -` then leaves the
    # session like any other cd outside it. The cd hook is not installed yet, so nothing fires.
    if [ -n "${VMR_SESSION_OLDPWD-}" ] && [ "${VMR_SESSION_OLDPWD}" != "${PWD}" ]; then
        _vmr_session_origin="${PWD}"
        if builtin cd -- "${VMR_SESSION_OLDPWD}" 2>/dev/null; then
            builtin cd -- "${_vmr_session_origin}" 2>/dev/null
        fi
        unset _vmr_session_origin
    fi
    _vmr_session_cd_hook() {
        VMR_CDHOOK_LAST_PWD=${PWD}
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

    # `z`/`zi` jump with `builtin cd`, bypassing the alias; watch PWD as well so leaving the
    # project that way still ends the session (the destination travels via the exit file).
    _vmr_pwd_watch() {
        local _st=$?
        if [ "${PWD}" != "${VMR_CDHOOK_LAST_PWD-}" ]; then
            VMR_CDHOOK_LAST_PWD=${PWD}
            _vmr_session_cd_hook
        fi
        return "${_st}"
    }

    if [ -n "${ZSH_VERSION-}" ]; then
        autoload -Uz add-zsh-hook
        add-zsh-hook chpwd _vmr_pwd_watch
    else
        case " ${PROMPT_COMMAND[*]-} " in
            *" _vmr_pwd_watch "*) ;;
            *)
                case "$(declare -p PROMPT_COMMAND 2>/dev/null)" in
                    "declare -a PROMPT_COMMAND="*) PROMPT_COMMAND=(_vmr_pwd_watch "${PROMPT_COMMAND[@]}") ;;
                    *) PROMPT_COMMAND="_vmr_pwd_watch${PROMPT_COMMAND:+;${PROMPT_COMMAND}}" ;;
                esac
                ;;
        esac
    fi
fi
