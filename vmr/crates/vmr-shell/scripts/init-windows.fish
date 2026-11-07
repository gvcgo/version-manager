if not set -q VM_DISABLE
    if test -f "__VMR_ENV_FILE__"
        . "__VMR_ENV_FILE__"
    end
__VMR_HOOK__
end

function svmr --description "leave a vmr session and reload the shell config"
    set -gx VM_DISABLE ''
    source "__VMR_CONF__"
end
