__VMR_SESSION_HOOK__
if [ -z "$VM_DISABLE" ]; then
    [ -f "__VMR_ENV_FILE__" ] && . "__VMR_ENV_FILE__"
__VMR_HOOK__
fi
alias svmr='export VM_DISABLE="" && source "__VMR_CONF__"'
