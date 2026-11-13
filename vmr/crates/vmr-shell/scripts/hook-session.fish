# `.vmr.lock` session: `cd` outside the project ends the child shell. The destination is
# recorded so the outer cd hook can move the parent shell there once the session exits;
# otherwise the user would land back at the project root.
if set -q VM_DISABLE; and set -q VMR_SESSION_PROJECT; and not set -q VMR_SESSION_ACTIVE
    set -gx VMR_SESSION_ACTIVE 1

    function _vmr_cd_hook
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
        _vmr_cd_hook
    end
end
