if [ "$funcstack[1]" = "_vmr" ]; then
    _vmr "$@"
elif (( $+functions[compdef] )); then
    compdef _vmr vmr
else
    autoload -Uz add-zsh-hook
    _vmr_register_completion() {
        if ! (( $+functions[compdef] )) && (( ! ${+_vmr_compinit_tried} )); then
            typeset -g _vmr_compinit_tried=1
            autoload -Uz compinit && compinit -i
        fi
        if (( $+functions[compdef] )); then
            compdef _vmr vmr
            add-zsh-hook -d precmd _vmr_register_completion
        fi
    }
    add-zsh-hook precmd _vmr_register_completion
fi