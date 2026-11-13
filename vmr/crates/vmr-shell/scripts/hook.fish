fish_add_path --global "__VMR_INSTALL__"

# `vmr cdhook` walks up from the working directory for `.vmr.lock` and loads the Lua plugins
# on every directory change. It records the applied project in the cache file below; this
# hook keeps that value in `VMR_CD_CACHE` and skips the call while the shell stays inside the
# same project. Leaving the project, or entering a directory that has its own `.vmr.lock`,
# looks it up again.
function _vmr_cd_hook
    set -l cache "$VMR_CD_CACHE"
    if test -n "$cache"; and test -f "$cache/.vmr.lock"
        if test "$PWD" = "$cache"
            return 0
        end
        if string match -q -- "$cache/*" "$PWD"; and not test -f "$PWD/.vmr.lock"
            return 0
        end
    end
    if type -q vmr
        vmr cdhook
    end
    set -gx VMR_CD_CACHE (cat "__VMR_CD_CACHE__" 2>/dev/null)
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
end

function _vmr_cdhook --on-variable="PWD" --description "version manager cd hook"
    _vmr_cd_hook
end

if not set -q VMR_CD_INIT
    set -gx VMR_CD_INIT "vmr_cd_init"
    _vmr_cd_hook
end
