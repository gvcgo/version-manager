fish_add_path --global "__VMR_INSTALL__"

# `vmr cdhook` walks up from the working directory for `.vmr.lock` and loads the Lua plugins
# on every directory change. It records the applied lock directory in the file below; this
# hook keeps that value in `VMR_LOCKFILE_DIR` and skips the call while the shell stays inside
# the same project. Leaving the project, or entering a directory that has its own
# `.vmr.lock`, looks it up again.
function _vmr_cd_hook
    set -l lockdir "$VMR_LOCKFILE_DIR"
    if test -n "$lockdir"; and test -f "$lockdir/.vmr.lock"
        if test "$PWD" = "$lockdir"
            return 0
        end
        if string match -q -- "$lockdir/*" "$PWD"; and not test -f "$PWD/.vmr.lock"
            return 0
        end
    end
    if type -q vmr
        # `cd -` in the session shell walks fish's `dirprev`; hand the current previous
        # directory over so the session hook can seed it.
        if set -q dirprev[-1]
            VMR_SESSION_OLDPWD=$dirprev[-1] vmr cdhook
        else
            vmr cdhook
        end
    end
    set -gx VMR_LOCKFILE_DIR (cat "__VMR_LOCKFILE_DIR__" 2>/dev/null)
    # A `.vmr.lock` session that ended because `cd` left its project records the destination;
    # move this shell there before looking up the new directory (it may have its own lock).
    if test -f "__VMR_SESSION_EXIT__"
        set -l dir (cat "__VMR_SESSION_EXIT__" 2>/dev/null)
        command rm -f "__VMR_SESSION_EXIT__"
        if test -n "$dir"; and test "$dir" != "$PWD"
            if cd -- "$dir"
                _vmr_cd_hook
                return
            end
        end
    end
    set -g VMR_CDHOOK_LAST_PWD $PWD
end

function _vmr_cdhook --on-variable="PWD" --description "version manager cd hook"
    # `z`/`zi` assign PWD without going through a `cd` alias, and this handler can change the
    # directory itself (session exit), which re-enters the event: skip both.
    if set -q VMR_CDHOOK_BUSY; or test "$PWD" = "$VMR_CDHOOK_LAST_PWD"
        return
    end
    set -gx VMR_CDHOOK_BUSY 1
    set -g VMR_CDHOOK_LAST_PWD $PWD
    _vmr_cd_hook
    set -e VMR_CDHOOK_BUSY
end

if not set -q VMR_CDHOOK_ACTIVE
    set -gx VMR_CDHOOK_ACTIVE 1
    _vmr_cd_hook
end
