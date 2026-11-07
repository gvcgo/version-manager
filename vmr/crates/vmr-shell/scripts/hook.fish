fish_add_path --global "__VMR_INSTALL__"

function _vmr_cdhook --on-variable="PWD" --description "version manager cd hook"
    if type -q vmr
        vmr cdhook
    end
end

if set -q "$VMR_CD_INIT"
    set VMR_CD_INIT "vmr_cd_init"
    cd "$(pwd)"
end
