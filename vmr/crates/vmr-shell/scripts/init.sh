__VMR_SESSION_HOOK__
if [ -z "$VM_DISABLE" ]; then
__VMR_REFRESH__
__VMR_WRAPPER__
    _vmr_refresh
__VMR_HOOK__
fi
alias svmr='export VM_DISABLE="" && source "__VMR_CONF__"'
