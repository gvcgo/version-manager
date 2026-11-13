# `.vmr.lock` session: `cd` outside the project ends the child shell. The destination is
# recorded so the outer cd hook can move the parent shell there once the session exits;
# otherwise the user would land back at the project root.
if set -q VM_DISABLE; and set -q VMR_SESSION_PROJECT; and not set -q VMR_SESSION_ACTIVE
    set -gx VMR_SESSION_ACTIVE 1
    # fish's `cd -` walks `dirprev`, which a fresh shell starts empty; seed it with the
    # directory the user jumped from so `cd -` leaves the session.
    if set -q VMR_SESSION_OLDPWD; and test -n "$VMR_SESSION_OLDPWD"
        set -g dirprev $VMR_SESSION_OLDPWD
    end

    function _vmr_cd_hook
        set -g VMR_CDHOOK_LAST_PWD $PWD
        set -l p (pwd -P)
        set -l project "$VMR_SESSION_PROJECT"
        # Compare in POSIX form: git-bash receives native Windows paths, native fish may not.
        if type -q cygpath
            if not string match -q -- '/*' "$p"
                set p (cygpath -u "$p")
            end
            if not string match -q -- '/*' "$project"
                set project (cygpath -u "$project")
            end
        end
        if test "$p" = "$project"; or string match -q -- "$project/*" "$p"
            return 0
        end
        printf '%s\n' "$PWD" > "__VMR_SESSION_EXIT__"
        exit
    end

    function _vmr_cdhook --on-variable="PWD" --description "leave the .vmr.lock session"
        if set -q VMR_CDHOOK_BUSY; or test "$PWD" = "$VMR_CDHOOK_LAST_PWD"
            return
        end
        set -gx VMR_CDHOOK_BUSY 1
        set -g VMR_CDHOOK_LAST_PWD $PWD
        _vmr_cd_hook
        set -e VMR_CDHOOK_BUSY
    end
end
