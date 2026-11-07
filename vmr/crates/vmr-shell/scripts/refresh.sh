_vmr_refresh() {
    [ -n "${VM_DISABLE}" ] && return 0
    local _vmr_root="$HOME/.vmr" _vmr_versions _vmr_rest _vmr_item _vmr_keep=
    if [ -n "${VMR_SDK_INSTALLATION_DIR}" ]; then
        _vmr_versions="${VMR_SDK_INSTALLATION_DIR}/versions"
    else
        _vmr_versions="${_vmr_root}/versions"
    fi
    _vmr_rest="${PATH}:"
    while [ -n "${_vmr_rest}" ]; do
        _vmr_item="${_vmr_rest%%:*}"
        _vmr_rest="${_vmr_rest#*:}"
        case "${_vmr_item}" in
            "${_vmr_root}"|"${_vmr_versions}"|"${_vmr_versions}"/*) ;;
            *)
                if [ -z "${_vmr_keep}" ]; then
                    _vmr_keep="${_vmr_item}"
                else
                    _vmr_keep="${_vmr_keep}:${_vmr_item}"
                fi
                ;;
        esac
    done
    PATH="${_vmr_root}${_vmr_keep:+:${_vmr_keep}}"
    export PATH
    if [ -f "__VMR_ENV_KEYS__" ]; then
        while IFS= read -r _vmr_item; do
            case "${_vmr_item}" in
                ''|PATH) ;;
                *) unset "${_vmr_item}" 2>/dev/null || true ;;
            esac
        done < "__VMR_ENV_KEYS__"
    fi
    if [ -f "__VMR_ENV_FILE__" ]; then
        . "__VMR_ENV_FILE__"
    fi
    return 0
}
