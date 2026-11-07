function _vmr_refresh --description "reload vmr-managed environment into the current shell"
    if set -q VM_DISABLE; and test -n "$VM_DISABLE"
        return 0
    end
    set -l _vmr_versions "$HOME/.vmr/versions"
    if set -q VMR_SDK_INSTALLATION_DIR; and test -n "$VMR_SDK_INSTALLATION_DIR"
        set _vmr_versions "$VMR_SDK_INSTALLATION_DIR/versions"
    end
    set -l _vmr_keep
    for _vmr_item in $PATH
        if test "$_vmr_item" = "$_vmr_versions"; or string match -q -- "$_vmr_versions/*" "$_vmr_item"
            continue
        end
        set -a _vmr_keep $_vmr_item
    end
    if test (count $_vmr_keep) -gt 0
        set -gx PATH $_vmr_keep
    end
    # `fish_add_path` skips entries already in `fish_user_paths` even after PATH was reset,
    # so stale vmr entries have to leave that list too, or re-sourcing would not re-add them.
    set -l _vmr_user_keep
    if set -q fish_user_paths
        for _vmr_item in $fish_user_paths
            if test "$_vmr_item" = "$_vmr_versions"; or string match -q -- "$_vmr_versions/*" "$_vmr_item"
                continue
            end
            set -a _vmr_user_keep $_vmr_item
        end
    end
    if test (count $_vmr_user_keep) -gt 0
        set -g fish_user_paths $_vmr_user_keep
    else
        set -e -g fish_user_paths 2>/dev/null
    end
    if test -f "__VMR_ENV_KEYS__"
        for _vmr_item in (cat "__VMR_ENV_KEYS__")
            if test -n "$_vmr_item"; and test "$_vmr_item" != PATH
                set -e $_vmr_item 2>/dev/null
            end
        end
    end
    if test -f "__VMR_ENV_FILE__"
        source "__VMR_ENV_FILE__"
    end
end
