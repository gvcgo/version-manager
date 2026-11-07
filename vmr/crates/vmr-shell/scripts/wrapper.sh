vmr() {
    command vmr "$@"
    local _vmr_status=$?
    if [ "${_vmr_status}" -eq 0 ] && [ $# -gt 0 ]; then
        case "$1" in
            use|u|h|uninstall|uni|r) _vmr_refresh ;;
        esac
    fi
    return "${_vmr_status}"
}
