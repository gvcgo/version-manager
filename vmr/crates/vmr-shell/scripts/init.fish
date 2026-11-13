__VMR_SESSION_HOOK__
if not set -q VM_DISABLE
__VMR_REFRESH__
__VMR_WRAPPER__
    _vmr_refresh
__VMR_HOOK__
end

function svmr --description "leave a vmr session and reload the shell config"
    set -gx VM_DISABLE ''
    source "__VMR_CONF__"
end
