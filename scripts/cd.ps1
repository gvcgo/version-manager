function cdhook {
    $TRUE_FALSE=(Test-Path $args[0])
    if ( $TRUE_FALSE -eq "True" )
    {
        cd $args[0]
        vmr use -E
    }
}

Set-Alias cd cdhook