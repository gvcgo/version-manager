function vmr --description "vmr with automatic environment refresh"
    command vmr $argv
    set -l _vmr_status $status
    if test $_vmr_status -eq 0; and test (count $argv) -gt 0
        switch $argv[1]
            case use u h uninstall uni r
                _vmr_refresh
        end
    end
    return $_vmr_status
end
