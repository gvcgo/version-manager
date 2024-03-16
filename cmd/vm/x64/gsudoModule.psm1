# Set $gsudoVerbose=$false before importing this module to remove the verbose messages.
if ($null -eq $gsudoVerbose) { $gsudoVerbose = $true; }
# Set $gsudoVerbose=$false before importing this module to remove the gsudo auto-complete functionality.
if ($null -eq $gsudoAutoComplete) { $gsudoAutoComplete = $true; }

$c = @("function Invoke-Gsudo {")
$c += (Get-Content "$PSScriptRoot\Invoke-Gsudo.ps1")
$c += "}"
iex ($c -join "`n" | Out-String)

function gsudo {
    <#
.SYNOPSIS
gsudo is a sudo for windows. It allows to run a command/ScriptBlock with elevated permissions. If no command is specified, it starts an elevated Powershell session.
.DESCRIPTION
# Syntax:
gsudo [options] { ScriptBlock } [ScriptBlock arguments]

gsudo [-n|--new]             # Run command in a new window and dont wait until command exits
      [-w|--wait]            # If --new is specified it wait until it exits.
      [-d 'CMD command']     # To elevate a Win32 CMD command instead of a Powershell script
      [--integrity {i}]      # Run with integrity level [Low, Medium, High, System]
      [-s]                   # Run as `NT AUTHORITY\System` 
      [--ti]                 # Run as Trusted Installer
      [-u|--user {username}] # Run as specific user (prompts for password)
      [--loadProfile]        # Loads the user profile on the elevated Powershell instance before running {ScriptBlock}
      { ScriptBlock }        # Script to elevate
      [-args $argument1[..., $argumentN]] ; # Pass arguments to the ScriptBlock, available as $args[0], $args[1]...

The command to elevate will run in a different process, so it can't access the parent $variables and scope.

More details about gsudo can be found by running: gsudo -h

.EXAMPLE
gsudo { Get-Process }
This run the `Get-Process` command as an administrator.

.EXAMPLE
gsudo { Get-Process $args[0] } -args "WinLogon"
Example case passing parameters to the ScriptBlock.

.INPUTS
You can pipe an input object and will be received as $input in the elevated ScriptBlock.

"WinLogon" | gsudo.exe { Get-Process $input }

.OUTPUTS
The output is determined by the command that is run with gsudo.

.LINK
https://github.com/gerardog/gsudo
#>

    # Note: gsudo is a windows application. 
    # This wrapper only serves the purpose of:
    #  - Adding support for `gsudo !!` on Powershell
    #  - Adding support for `Get-Help gsudo`

    $invocationLine = $MyInvocation.Line -replace "^$($MyInvocation.InvocationName)\s+" # -replace '"','""'

    if ($invocationLine -match "(^| )!!( |$)") { 
        $i = 0;
        do {
            $c = (Get-History | Select-Object -last 1 -skip $i).CommandLine
            $i++;
        } while ($c -eq $MyInvocation.Line -and $c)
        
        if ($c) { 
            if ($gsudoVerbose) { Write-verbose "Elevating Command: '$c'" -Verbose }
            gsudo.exe $c 
        }
        else {
            throw "Failed to find last invoked command in Powershell history."
        }
    }
    elseif ($myinvocation.expectingInput) {
        $input | & gsudo.exe @args 
    } 
    else { 
        & gsudo.exe @args 
    }
}

function Test-IsGsudoCacheAvailable {
    return ('true' -eq (gsudo status CacheAvailable))
}

function Test-IsProcessElevated {
    <#
.Synopsis
    Tests if the user is an administrator *and* the current proces is elevated.
.Description
    Returns true if the current process is elevated.
.Example
    Test-IsAdmin
#>	
    if ($PSVersionTable.Platform -eq 'Unix') {
        return (id -u) -eq 0
    }
    else {
        $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = New-Object Security.Principal.WindowsPrincipal $identity
        return $principal.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
    }
}

function Test-IsAdminMember {
    <#
.SYNOPSIS
The function Test-IsAdminMember checks if the currently logged-in user is a member of the local administrators group, regardless of the elevation level of the current process.
#>
    $userName = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
    $adminGroupSid = "S-1-5-32-544"
    $localAdminGroup = Get-LocalGroup -SID $adminGroupSid
    $isAdmin = (Get-LocalGroupMember -Group $localAdminGroup.Name).Where({ $_.Name -eq $userName }).Count -gt 0
    return $isAdmin
}

Function gsudoPrompt {
    $eol = If (Test-IsProcessElevated) { "$([char]27)[1;31m" + ('#') * ($nestedPromptLevel + 1) + "$([char]27)[0m" } else { '>' * ($nestedPromptLevel + 1) };
    "PS $($executionContext.SessionState.Path.CurrentLocation)$eol ";
}

if ($gsudoAutoComplete) {
    #Create an auto-completer for gsudo.

    $verbs = @('status', 'cache', 'config', 'help', '!!')
    $options = @('-d', '--loadProfile', '--system', '--ti', '-k', '--new', '--wait', '--keepShell', '--keepWindow', '--help', '--debug', '--copyNS', '--integrity', '--user')

    $integrityOptions = @("Low", "Medium", "MediumPlus", "High", "System")
    $TrueFalseReset = @('true', 'false', '--reset')

    $suggestions = @{ 
        '--integrity'                 = $integrityOptions;
        '-i'                          = $integrityOptions;
        'cache'                       = @('on', 'off', 'help');
        'config'                      = @('CacheMode', 'CacheDuration', 'LogLevel', 'NewWindow.Force', 'NewWindow.CloseBehaviour', 'Prompt', 'PipedPrompt', 'ForceAttachedConsole', 'ForcePipedConsole', 'ForceVTConsole', 'CopyEnvironmentVariables', 'CopyNetworkShares', 'PowerShellLoadProfile', 'SecurityEnforceUacIsolation', 'ExceptionList');		
        'cachemode'                   = @('Auto', 'Disabled', 'Explicit', '--reset');
        'loglevel'                    = @('All', 'Debug', 'Info', 'Warning', 'Error', 'None', '--reset');
        'NewWindow.CloseBehaviour'    = @('KeepShellOpen', 'PressKeyToClose', 'OsDefault', '--reset');
        'NewWindow.Force'             = $TrueFalseReset;
        'ForceAttachedConsole'        = $TrueFalseReset;
        'ForcePipedConsole'           = $TrueFalseReset;
        'ForceVTConsole'              = $TrueFalseReset;
        'CopyEnvironmentVariables'    = $TrueFalseReset;
        'CopyNetworkShares'           = $TrueFalseReset;
        'PowerShellLoadProfile'       = $TrueFalseReset;
        'SecurityEnforceUacIsolation' = $TrueFalseReset;
		'Status'                      = @('--json', 'CallerPid', 'UserName', 'UserSid', 'IsElevated', 'IsAdminMember', 'IntegrityLevelNumeric', 'IntegrityLevel', 'CacheMode', 'CacheAvailable', 'CacheSessionsCount', 'CacheSessions', 'IsRedirected', '--no-output')
        '--user'                      = @("$env:USERDOMAIN\$env:USERNAME");
        '-u'                          = @("$env:USERDOMAIN\$env:USERNAME")
    }

    $autoCompleter = {
        param($wordToComplete, $commandAst, $cursorPosition)
    
        # gsudo powershell syntax is:
        # gsudo [gsudo options] [optional-gsudo-verb] [gsudo-verb-options | command-to-elevate] [commant-to-elevate-args]
        
        # Will use $phase variable to signal which part of the command is being auto-completed.
        # Phase 1 means autocomplete for [options]
        # Phase 2 means autocomplete for [gsudo-verb]
        # Phase 3 means autocomplete for [verb-options]
        # Phase 4 means [command] is already written.

        $commands = $commandAst.ToString().Substring(0, $cursorPosition - 1).Split(' ') | select -Skip 1;
        if ($wordToComplete) {
            $lastWord = ($commands | select -Last 1 -skip 1)
        }
        else {
            $lastWord = ($commands | select -Last 1)
        }

<# Debugging aids
        # Save the current cursor position
        $originalX = $host.ui.RawUI.CursorPosition.X
        $originalY = $host.ui.RawUI.CursorPosition.Y
        
        # Set the cursor position to (0,0)
        $host.ui.RawUI.CursorPosition = New-Object System.Management.Automation.Host.Coordinates 0, 0
        
        Write-Debug -Debug "wordToComplete = ""$wordToComplete""         "
        Write-Debug -Debug "commandAst = ""$commandAst""         "
        Write-Debug -Debug "cursorPosition = ""$cursorPosition""         "
        Write-Debug -Debug "commands = ""$commands""     ";
        Write-Debug -Debug "lastWord = ""$lastWord""     ";
#>    
        $phase = 1;
    
        foreach ($c in $commands) {
            if ($phase -le 2) {
                if ($verbs -contains $c) { $phase = 3 }
                if ($c -like '{*') { $phase = 4 }
            }
        }

        $filter = "$wordToComplete*"
    
        if ($lastWord -and $suggestions[$lastWord]) {
            $suggestions[$lastWord] -like $filter | % { $_ }
        }
        else {
            if ($phase -lt 3) { 
                if ($wordToComplete -eq '') {
                    # Suggest last 3 executed commands.
                    $lastCommands = Get-History | Select-Object -last 3 | % { "{ $($_.CommandLine) }" }
                
                    if ($lastCommands -is [System.Array]) {
                        # Last one first.
                        $lastCommands[($lastCommands.Length - 1)..0] | % { $_ };
                    }
                    elseif ($lastCommands) {
                        # Only one command.
                        $lastCommands;
                    }
                }
            }
            if ($phase -le 2) { $verbs -like $filter; }	
            if ($phase -le 1) { $options -like $filter; }
            if ($phase -ge 4) { '-args' }

        }
<# Debugging aids
        Write-Debug -Debug "----";

        # Return the cursor position to its original location
        $host.ui.RawUI.CursorPosition = New-Object System.Management.Automation.Host.Coordinates $originalX, $originalY 
#>
    }

    Register-ArgumentCompleter -Native -CommandName 'gsudo' -ScriptBlock $autoCompleter
    Register-ArgumentCompleter -Native -CommandName 'sudo' -ScriptBlock $autoCompleter
}

Export-ModuleMember -function Invoke-Gsudo, gsudo, Test-IsGsudoCacheAvailable, Test-IsProcessElevated, Test-IsAdminMember, gsudoPrompt -Variable gsudoVerbose, gsudoAutoComplete
# SIG # Begin signature block
# MIIr1gYJKoZIhvcNAQcCoIIrxzCCK8MCAQExDzANBglghkgBZQMEAgEFADB5Bgor
# BgEEAYI3AgEEoGswaTA0BgorBgEEAYI3AgEeMCYCAwEAAAQQH8w7YFlLCE63JNLG
# KX7zUQIBAAIBAAIBAAIBAAIBADAxMA0GCWCGSAFlAwQCAQUABCDUPXhBSTdC17OD
# DHdMiVBOxTYvFAyhrkzeAADYGfj1ZKCCJPYwggVvMIIEV6ADAgECAhBI/JO0YFWU
# jTanyYqJ1pQWMA0GCSqGSIb3DQEBDAUAMHsxCzAJBgNVBAYTAkdCMRswGQYDVQQI
# DBJHcmVhdGVyIE1hbmNoZXN0ZXIxEDAOBgNVBAcMB1NhbGZvcmQxGjAYBgNVBAoM
# EUNvbW9kbyBDQSBMaW1pdGVkMSEwHwYDVQQDDBhBQUEgQ2VydGlmaWNhdGUgU2Vy
# dmljZXMwHhcNMjEwNTI1MDAwMDAwWhcNMjgxMjMxMjM1OTU5WjBWMQswCQYDVQQG
# EwJHQjEYMBYGA1UEChMPU2VjdGlnbyBMaW1pdGVkMS0wKwYDVQQDEyRTZWN0aWdv
# IFB1YmxpYyBDb2RlIFNpZ25pbmcgUm9vdCBSNDYwggIiMA0GCSqGSIb3DQEBAQUA
# A4ICDwAwggIKAoICAQCN55QSIgQkdC7/FiMCkoq2rjaFrEfUI5ErPtx94jGgUW+s
# hJHjUoq14pbe0IdjJImK/+8Skzt9u7aKvb0Ffyeba2XTpQxpsbxJOZrxbW6q5KCD
# J9qaDStQ6Utbs7hkNqR+Sj2pcaths3OzPAsM79szV+W+NDfjlxtd/R8SPYIDdub7
# P2bSlDFp+m2zNKzBenjcklDyZMeqLQSrw2rq4C+np9xu1+j/2iGrQL+57g2extme
# me/G3h+pDHazJyCh1rr9gOcB0u/rgimVcI3/uxXP/tEPNqIuTzKQdEZrRzUTdwUz
# T2MuuC3hv2WnBGsY2HH6zAjybYmZELGt2z4s5KoYsMYHAXVn3m3pY2MeNn9pib6q
# RT5uWl+PoVvLnTCGMOgDs0DGDQ84zWeoU4j6uDBl+m/H5x2xg3RpPqzEaDux5mcz
# mrYI4IAFSEDu9oJkRqj1c7AGlfJsZZ+/VVscnFcax3hGfHCqlBuCF6yH6bbJDoEc
# QNYWFyn8XJwYK+pF9e+91WdPKF4F7pBMeufG9ND8+s0+MkYTIDaKBOq3qgdGnA2T
# OglmmVhcKaO5DKYwODzQRjY1fJy67sPV+Qp2+n4FG0DKkjXp1XrRtX8ArqmQqsV/
# AZwQsRb8zG4Y3G9i/qZQp7h7uJ0VP/4gDHXIIloTlRmQAOka1cKG8eOO7F/05QID
# AQABo4IBEjCCAQ4wHwYDVR0jBBgwFoAUoBEKIz6W8Qfs4q8p74Klf9AwpLQwHQYD
# VR0OBBYEFDLrkpr/NZZILyhAQnAgNpFcF4XmMA4GA1UdDwEB/wQEAwIBhjAPBgNV
# HRMBAf8EBTADAQH/MBMGA1UdJQQMMAoGCCsGAQUFBwMDMBsGA1UdIAQUMBIwBgYE
# VR0gADAIBgZngQwBBAEwQwYDVR0fBDwwOjA4oDagNIYyaHR0cDovL2NybC5jb21v
# ZG9jYS5jb20vQUFBQ2VydGlmaWNhdGVTZXJ2aWNlcy5jcmwwNAYIKwYBBQUHAQEE
# KDAmMCQGCCsGAQUFBzABhhhodHRwOi8vb2NzcC5jb21vZG9jYS5jb20wDQYJKoZI
# hvcNAQEMBQADggEBABK/oe+LdJqYRLhpRrWrJAoMpIpnuDqBv0WKfVIHqI0fTiGF
# OaNrXi0ghr8QuK55O1PNtPvYRL4G2VxjZ9RAFodEhnIq1jIV9RKDwvnhXRFAZ/ZC
# J3LFI+ICOBpMIOLbAffNRk8monxmwFE2tokCVMf8WPtsAO7+mKYulaEMUykfb9gZ
# pk+e96wJ6l2CxouvgKe9gUhShDHaMuwV5KZMPWw5c9QLhTkg4IUaaOGnSDip0TYl
# d8GNGRbFiExmfS9jzpjoad+sPKhdnckcW67Y8y90z7h+9teDnRGWYpquRRPaf9xH
# +9/DUp/mBlXpnYzyOmJRvOwkDynUWICE5EV7WtgwggWNMIIEdaADAgECAhAOmxiO
# +dAt5+/bUOIIQBhaMA0GCSqGSIb3DQEBDAUAMGUxCzAJBgNVBAYTAlVTMRUwEwYD
# VQQKEwxEaWdpQ2VydCBJbmMxGTAXBgNVBAsTEHd3dy5kaWdpY2VydC5jb20xJDAi
# BgNVBAMTG0RpZ2lDZXJ0IEFzc3VyZWQgSUQgUm9vdCBDQTAeFw0yMjA4MDEwMDAw
# MDBaFw0zMTExMDkyMzU5NTlaMGIxCzAJBgNVBAYTAlVTMRUwEwYDVQQKEwxEaWdp
# Q2VydCBJbmMxGTAXBgNVBAsTEHd3dy5kaWdpY2VydC5jb20xITAfBgNVBAMTGERp
# Z2lDZXJ0IFRydXN0ZWQgUm9vdCBHNDCCAiIwDQYJKoZIhvcNAQEBBQADggIPADCC
# AgoCggIBAL/mkHNo3rvkXUo8MCIwaTPswqclLskhPfKK2FnC4SmnPVirdprNrnsb
# hA3EMB/zG6Q4FutWxpdtHauyefLKEdLkX9YFPFIPUh/GnhWlfr6fqVcWWVVyr2iT
# cMKyunWZanMylNEQRBAu34LzB4TmdDttceItDBvuINXJIB1jKS3O7F5OyJP4IWGb
# NOsFxl7sWxq868nPzaw0QF+xembud8hIqGZXV59UWI4MK7dPpzDZVu7Ke13jrclP
# XuU15zHL2pNe3I6PgNq2kZhAkHnDeMe2scS1ahg4AxCN2NQ3pC4FfYj1gj4QkXCr
# VYJBMtfbBHMqbpEBfCFM1LyuGwN1XXhm2ToxRJozQL8I11pJpMLmqaBn3aQnvKFP
# ObURWBf3JFxGj2T3wWmIdph2PVldQnaHiZdpekjw4KISG2aadMreSx7nDmOu5tTv
# kpI6nj3cAORFJYm2mkQZK37AlLTSYW3rM9nF30sEAMx9HJXDj/chsrIRt7t/8tWM
# cCxBYKqxYxhElRp2Yn72gLD76GSmM9GJB+G9t+ZDpBi4pncB4Q+UDCEdslQpJYls
# 5Q5SUUd0viastkF13nqsX40/ybzTQRESW+UQUOsxxcpyFiIJ33xMdT9j7CFfxCBR
# a2+xq4aLT8LWRV+dIPyhHsXAj6KxfgommfXkaS+YHS312amyHeUbAgMBAAGjggE6
# MIIBNjAPBgNVHRMBAf8EBTADAQH/MB0GA1UdDgQWBBTs1+OC0nFdZEzfLmc/57qY
# rhwPTzAfBgNVHSMEGDAWgBRF66Kv9JLLgjEtUYunpyGd823IDzAOBgNVHQ8BAf8E
# BAMCAYYweQYIKwYBBQUHAQEEbTBrMCQGCCsGAQUFBzABhhhodHRwOi8vb2NzcC5k
# aWdpY2VydC5jb20wQwYIKwYBBQUHMAKGN2h0dHA6Ly9jYWNlcnRzLmRpZ2ljZXJ0
# LmNvbS9EaWdpQ2VydEFzc3VyZWRJRFJvb3RDQS5jcnQwRQYDVR0fBD4wPDA6oDig
# NoY0aHR0cDovL2NybDMuZGlnaWNlcnQuY29tL0RpZ2lDZXJ0QXNzdXJlZElEUm9v
# dENBLmNybDARBgNVHSAECjAIMAYGBFUdIAAwDQYJKoZIhvcNAQEMBQADggEBAHCg
# v0NcVec4X6CjdBs9thbX979XB72arKGHLOyFXqkauyL4hxppVCLtpIh3bb0aFPQT
# SnovLbc47/T/gLn4offyct4kvFIDyE7QKt76LVbP+fT3rDB6mouyXtTP0UNEm0Mh
# 65ZyoUi0mcudT6cGAxN3J0TU53/oWajwvy8LpunyNDzs9wPHh6jSTEAZNUZqaVSw
# uKFWjuyk1T3osdz9HNj0d1pcVIxv76FQPfx2CWiEn2/K2yCNNWAcAgPLILCsWKAO
# QGPFmCLBsln1VWvPJ6tsds5vIy30fnFqI2si/xK4VC0nftg62fC2h5b9W9FcrBjD
# TZ9ztwGpn1eqXijiuZQwggYaMIIEAqADAgECAhBiHW0MUgGeO5B5FSCJIRwKMA0G
# CSqGSIb3DQEBDAUAMFYxCzAJBgNVBAYTAkdCMRgwFgYDVQQKEw9TZWN0aWdvIExp
# bWl0ZWQxLTArBgNVBAMTJFNlY3RpZ28gUHVibGljIENvZGUgU2lnbmluZyBSb290
# IFI0NjAeFw0yMTAzMjIwMDAwMDBaFw0zNjAzMjEyMzU5NTlaMFQxCzAJBgNVBAYT
# AkdCMRgwFgYDVQQKEw9TZWN0aWdvIExpbWl0ZWQxKzApBgNVBAMTIlNlY3RpZ28g
# UHVibGljIENvZGUgU2lnbmluZyBDQSBSMzYwggGiMA0GCSqGSIb3DQEBAQUAA4IB
# jwAwggGKAoIBgQCbK51T+jU/jmAGQ2rAz/V/9shTUxjIztNsfvxYB5UXeWUzCxEe
# AEZGbEN4QMgCsJLZUKhWThj/yPqy0iSZhXkZ6Pg2A2NVDgFigOMYzB2OKhdqfWGV
# oYW3haT29PSTahYkwmMv0b/83nbeECbiMXhSOtbam+/36F09fy1tsB8je/RV0mIk
# 8XL/tfCK6cPuYHE215wzrK0h1SWHTxPbPuYkRdkP05ZwmRmTnAO5/arnY83jeNzh
# P06ShdnRqtZlV59+8yv+KIhE5ILMqgOZYAENHNX9SJDm+qxp4VqpB3MV/h53yl41
# aHU5pledi9lCBbH9JeIkNFICiVHNkRmq4TpxtwfvjsUedyz8rNyfQJy/aOs5b4s+
# ac7IH60B+Ja7TVM+EKv1WuTGwcLmoU3FpOFMbmPj8pz44MPZ1f9+YEQIQty/NQd/
# 2yGgW+ufflcZ/ZE9o1M7a5Jnqf2i2/uMSWymR8r2oQBMdlyh2n5HirY4jKnFH/9g
# Rvd+QOfdRrJZb1sCAwEAAaOCAWQwggFgMB8GA1UdIwQYMBaAFDLrkpr/NZZILyhA
# QnAgNpFcF4XmMB0GA1UdDgQWBBQPKssghyi47G9IritUpimqF6TNDDAOBgNVHQ8B
# Af8EBAMCAYYwEgYDVR0TAQH/BAgwBgEB/wIBADATBgNVHSUEDDAKBggrBgEFBQcD
# AzAbBgNVHSAEFDASMAYGBFUdIAAwCAYGZ4EMAQQBMEsGA1UdHwREMEIwQKA+oDyG
# Omh0dHA6Ly9jcmwuc2VjdGlnby5jb20vU2VjdGlnb1B1YmxpY0NvZGVTaWduaW5n
# Um9vdFI0Ni5jcmwwewYIKwYBBQUHAQEEbzBtMEYGCCsGAQUFBzAChjpodHRwOi8v
# Y3J0LnNlY3RpZ28uY29tL1NlY3RpZ29QdWJsaWNDb2RlU2lnbmluZ1Jvb3RSNDYu
# cDdjMCMGCCsGAQUFBzABhhdodHRwOi8vb2NzcC5zZWN0aWdvLmNvbTANBgkqhkiG
# 9w0BAQwFAAOCAgEABv+C4XdjNm57oRUgmxP/BP6YdURhw1aVcdGRP4Wh60BAscjW
# 4HL9hcpkOTz5jUug2oeunbYAowbFC2AKK+cMcXIBD0ZdOaWTsyNyBBsMLHqafvIh
# rCymlaS98+QpoBCyKppP0OcxYEdU0hpsaqBBIZOtBajjcw5+w/KeFvPYfLF/ldYp
# mlG+vd0xqlqd099iChnyIMvY5HexjO2AmtsbpVn0OhNcWbWDRF/3sBp6fWXhz7Dc
# ML4iTAWS+MVXeNLj1lJziVKEoroGs9Mlizg0bUMbOalOhOfCipnx8CaLZeVme5yE
# Lg09Jlo8BMe80jO37PU8ejfkP9/uPak7VLwELKxAMcJszkyeiaerlphwoKx1uHRz
# NyE6bxuSKcutisqmKL5OTunAvtONEoteSiabkPVSZ2z76mKnzAfZxCl/3dq3dUNw
# 4rg3sTCggkHSRqTqlLMS7gjrhTqBmzu1L90Y1KWN/Y5JKdGvspbOrTfOXyXvmPL6
# E52z1NZJ6ctuMFBQZH3pwWvqURR8AgQdULUvrxjUYbHHj95Ejza63zdrEcxWLDX6
# xWls/GDnVNueKjWUH3fTv1Y8Wdho698YADR7TNx8X8z2Bev6SivBBOHY+uqiirZt
# g0y9ShQoPzmCcn63Syatatvx157YK9hlcPmVoa1oDE5/L9Uo2bC5a4CH2RwwggZY
# MIIEwKADAgECAhEA1hBezo41z3AItTU5kYK/yTANBgkqhkiG9w0BAQwFADBUMQsw
# CQYDVQQGEwJHQjEYMBYGA1UEChMPU2VjdGlnbyBMaW1pdGVkMSswKQYDVQQDEyJT
# ZWN0aWdvIFB1YmxpYyBDb2RlIFNpZ25pbmcgQ0EgUjM2MB4XDTIyMTEyMTAwMDAw
# MFoXDTI1MTEyMDIzNTk1OVowbjELMAkGA1UEBhMCQVIxKTAnBgNVBAgMIENpdWRh
# ZCBBdXTDs25vbWEgZGUgQnVlbm9zIEFpcmVzMRkwFwYDVQQKDBBHZXJhcmRvIEdy
# aWdub2xpMRkwFwYDVQQDDBBHZXJhcmRvIEdyaWdub2xpMIICIjANBgkqhkiG9w0B
# AQEFAAOCAg8AMIICCgKCAgEAt/W5DVIya5ejfBByJc33Y7MWCBQnisri6c5ybt81
# lPUPg3i8jfaOg6YOOFvmhRDgM49sTXWK3rkjRHrCnWKVKDb8i2hiU6dHc4Ra7nos
# i5ipmhJgvhJVLzWxTxEyrjixBIpUm6XKPCArWrancVAWotCi6kyB/+RL0OLlXzQd
# kx8a4/9Ub27WEvbn6u66/Idv/hipDuHSpM80RuspV7J08RHbIdZBUY1kU9itjs/u
# BsCSSheqvlvIQzfl1CmXv1KtfjBowHYS2o6OQmVyKRPg8K9O3ZwvL8uJMwxfOcT7
# 5hn3ffEwxnbOvHEBeiE851A+bW1LBc+x++8A3K6ZhHLmmhIsgg+77ujx9Z9EzaNB
# CStbq/SHNfRQjBFWS+jfXofppLREenUjwuDNdgHsbpeNh0YZgUsri8K81EIrnIOw
# yyQfIlGYFLWfNwIQATzZralA/Z3BJAEW0rKGXu8FtBw2QKRcj5kDE3eEoU8wAZEU
# JolVgBXeDV9gygAgPVvi9r/8WPJiyZgAFzF0zd+sIci5aDyKqtc82cZflRi5uzf+
# emLYy7grtkHXbJ9XeSF87JGIHP2ryJiYQbxmBV4XpI4unANAU7RHdXKlWElkQ58O
# 5P4o4RlVGc/bAlREml7Rl5De/T8KpjhxR5VhnTeZNax2G0DVTD7Wsbxj0TmAnASt
# ZNsCAwEAAaOCAYkwggGFMB8GA1UdIwQYMBaAFA8qyyCHKLjsb0iuK1SmKaoXpM0M
# MB0GA1UdDgQWBBQUXpGJwKTGIAYQuTTSFpz1f2aFIjAOBgNVHQ8BAf8EBAMCB4Aw
# DAYDVR0TAQH/BAIwADATBgNVHSUEDDAKBggrBgEFBQcDAzBKBgNVHSAEQzBBMDUG
# DCsGAQQBsjEBAgEDAjAlMCMGCCsGAQUFBwIBFhdodHRwczovL3NlY3RpZ28uY29t
# L0NQUzAIBgZngQwBBAEwSQYDVR0fBEIwQDA+oDygOoY4aHR0cDovL2NybC5zZWN0
# aWdvLmNvbS9TZWN0aWdvUHVibGljQ29kZVNpZ25pbmdDQVIzNi5jcmwweQYIKwYB
# BQUHAQEEbTBrMEQGCCsGAQUFBzAChjhodHRwOi8vY3J0LnNlY3RpZ28uY29tL1Nl
# Y3RpZ29QdWJsaWNDb2RlU2lnbmluZ0NBUjM2LmNydDAjBggrBgEFBQcwAYYXaHR0
# cDovL29jc3Auc2VjdGlnby5jb20wDQYJKoZIhvcNAQEMBQADggGBABOIZQVhJqkO
# koVyJKjlc8sMtBWiTWvIZqggyvR1FeOFBYkHOQxd9CWiRhqSbGeE55n1+JfYhhCF
# 0zHK7lMouZKa8rxYW5yOufTX2sJIsNYmfnpyD9SJRgloAPaxjB5pu0ZJ9Yx84wyW
# /DO2t3Vn3myPjW3wPdfS0GfN5BJvtykT/fxyakZ3pi9S8AvwCJSG/qWeOzjjj9Bv
# LTKfHE5ivz5Y6Hyqh/LsSjXsijXgNbcvoEPjBYtTdFc8kDS+kZtydmORKGnMebag
# JTdC+Lh+yZdY1F+2XEIQpYHz+x4kJVEQhjV7g0PPdNVcF/zjU2J53SQ66+SW9yvj
# j2aixrID4czk176IBFQ/1O2+I+rU7OQ+HfTwsH0mE7GOgp33gQSOhJXMHTnIy62J
# pdJEHOnUINPvxcnoOxajDXQ9IjRyQZN1soW2GAPI4/2+Zu1NsxX3sjDNcgy1+zn4
# MpQe25SrBGo7WpSwfNRk01CuaVbjz9rWP0kzFA+P2Mgsl2GsFSRiITCCBq4wggSW
# oAMCAQICEAc2N7ckVHzYR6z9KGYqXlswDQYJKoZIhvcNAQELBQAwYjELMAkGA1UE
# BhMCVVMxFTATBgNVBAoTDERpZ2lDZXJ0IEluYzEZMBcGA1UECxMQd3d3LmRpZ2lj
# ZXJ0LmNvbTEhMB8GA1UEAxMYRGlnaUNlcnQgVHJ1c3RlZCBSb290IEc0MB4XDTIy
# MDMyMzAwMDAwMFoXDTM3MDMyMjIzNTk1OVowYzELMAkGA1UEBhMCVVMxFzAVBgNV
# BAoTDkRpZ2lDZXJ0LCBJbmMuMTswOQYDVQQDEzJEaWdpQ2VydCBUcnVzdGVkIEc0
# IFJTQTQwOTYgU0hBMjU2IFRpbWVTdGFtcGluZyBDQTCCAiIwDQYJKoZIhvcNAQEB
# BQADggIPADCCAgoCggIBAMaGNQZJs8E9cklRVcclA8TykTepl1Gh1tKD0Z5Mom2g
# sMyD+Vr2EaFEFUJfpIjzaPp985yJC3+dH54PMx9QEwsmc5Zt+FeoAn39Q7SE2hHx
# c7Gz7iuAhIoiGN/r2j3EF3+rGSs+QtxnjupRPfDWVtTnKC3r07G1decfBmWNlCnT
# 2exp39mQh0YAe9tEQYncfGpXevA3eZ9drMvohGS0UvJ2R/dhgxndX7RUCyFobjch
# u0CsX7LeSn3O9TkSZ+8OpWNs5KbFHc02DVzV5huowWR0QKfAcsW6Th+xtVhNef7X
# j3OTrCw54qVI1vCwMROpVymWJy71h6aPTnYVVSZwmCZ/oBpHIEPjQ2OAe3VuJyWQ
# mDo4EbP29p7mO1vsgd4iFNmCKseSv6De4z6ic/rnH1pslPJSlRErWHRAKKtzQ87f
# SqEcazjFKfPKqpZzQmiftkaznTqj1QPgv/CiPMpC3BhIfxQ0z9JMq++bPf4OuGQq
# +nUoJEHtQr8FnGZJUlD0UfM2SU2LINIsVzV5K6jzRWC8I41Y99xh3pP+OcD5sjCl
# TNfpmEpYPtMDiP6zj9NeS3YSUZPJjAw7W4oiqMEmCPkUEBIDfV8ju2TjY+Cm4T72
# wnSyPx4JduyrXUZ14mCjWAkBKAAOhFTuzuldyF4wEr1GnrXTdrnSDmuZDNIztM2x
# AgMBAAGjggFdMIIBWTASBgNVHRMBAf8ECDAGAQH/AgEAMB0GA1UdDgQWBBS6Ftlt
# TYUvcyl2mi91jGogj57IbzAfBgNVHSMEGDAWgBTs1+OC0nFdZEzfLmc/57qYrhwP
# TzAOBgNVHQ8BAf8EBAMCAYYwEwYDVR0lBAwwCgYIKwYBBQUHAwgwdwYIKwYBBQUH
# AQEEazBpMCQGCCsGAQUFBzABhhhodHRwOi8vb2NzcC5kaWdpY2VydC5jb20wQQYI
# KwYBBQUHMAKGNWh0dHA6Ly9jYWNlcnRzLmRpZ2ljZXJ0LmNvbS9EaWdpQ2VydFRy
# dXN0ZWRSb290RzQuY3J0MEMGA1UdHwQ8MDowOKA2oDSGMmh0dHA6Ly9jcmwzLmRp
# Z2ljZXJ0LmNvbS9EaWdpQ2VydFRydXN0ZWRSb290RzQuY3JsMCAGA1UdIAQZMBcw
# CAYGZ4EMAQQCMAsGCWCGSAGG/WwHATANBgkqhkiG9w0BAQsFAAOCAgEAfVmOwJO2
# b5ipRCIBfmbW2CFC4bAYLhBNE88wU86/GPvHUF3iSyn7cIoNqilp/GnBzx0H6T5g
# yNgL5Vxb122H+oQgJTQxZ822EpZvxFBMYh0MCIKoFr2pVs8Vc40BIiXOlWk/R3f7
# cnQU1/+rT4osequFzUNf7WC2qk+RZp4snuCKrOX9jLxkJodskr2dfNBwCnzvqLx1
# T7pa96kQsl3p/yhUifDVinF2ZdrM8HKjI/rAJ4JErpknG6skHibBt94q6/aesXmZ
# gaNWhqsKRcnfxI2g55j7+6adcq/Ex8HBanHZxhOACcS2n82HhyS7T6NJuXdmkfFy
# nOlLAlKnN36TU6w7HQhJD5TNOXrd/yVjmScsPT9rp/Fmw0HNT7ZAmyEhQNC3EyTN
# 3B14OuSereU0cZLXJmvkOHOrpgFPvT87eK1MrfvElXvtCl8zOYdBeHo46Zzh3SP9
# HSjTx/no8Zhf+yvYfvJGnXUsHicsJttvFXseGYs2uJPU5vIXmVnKcPA3v5gA3yAW
# Tyf7YGcWoWa63VXAOimGsJigK+2VQbc61RWYMbRiCQ8KvYHZE/6/pNHzV9m8BPqC
# 3jLfBInwAM1dwvnQI38AC+R2AibZ8GV2QqYphwlHK+Z/GqSFD/yYlvZVVCsfgPrA
# 8g4r5db7qS9EFUrnEw4d2zc4GqEr9u3WfPwwggbCMIIEqqADAgECAhAFRK/zlJ0I
# Oaa/2z9f5WEWMA0GCSqGSIb3DQEBCwUAMGMxCzAJBgNVBAYTAlVTMRcwFQYDVQQK
# Ew5EaWdpQ2VydCwgSW5jLjE7MDkGA1UEAxMyRGlnaUNlcnQgVHJ1c3RlZCBHNCBS
# U0E0MDk2IFNIQTI1NiBUaW1lU3RhbXBpbmcgQ0EwHhcNMjMwNzE0MDAwMDAwWhcN
# MzQxMDEzMjM1OTU5WjBIMQswCQYDVQQGEwJVUzEXMBUGA1UEChMORGlnaUNlcnQs
# IEluYy4xIDAeBgNVBAMTF0RpZ2lDZXJ0IFRpbWVzdGFtcCAyMDIzMIICIjANBgkq
# hkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAo1NFhx2DjlusPlSzI+DPn9fl0uddoQ4J
# 3C9Io5d6OyqcZ9xiFVjBqZMRp82qsmrdECmKHmJjadNYnDVxvzqX65RQjxwg6sea
# Oy+WZuNp52n+W8PWKyAcwZeUtKVQgfLPywemMGjKg0La/H8JJJSkghraarrYO8pd
# 3hkYhftF6g1hbJ3+cV7EBpo88MUueQ8bZlLjyNY+X9pD04T10Mf2SC1eRXWWdf7d
# EKEbg8G45lKVtUfXeCk5a+B4WZfjRCtK1ZXO7wgX6oJkTf8j48qG7rSkIWRw69Xl
# oNpjsy7pBe6q9iT1HbybHLK3X9/w7nZ9MZllR1WdSiQvrCuXvp/k/XtzPjLuUjT7
# 1Lvr1KAsNJvj3m5kGQc3AZEPHLVRzapMZoOIaGK7vEEbeBlt5NkP4FhB+9ixLOFR
# r7StFQYU6mIIE9NpHnxkTZ0P387RXoyqq1AVybPKvNfEO2hEo6U7Qv1zfe7dCv95
# NBB+plwKWEwAPoVpdceDZNZ1zY8SdlalJPrXxGshuugfNJgvOuprAbD3+yqG7HtS
# OKmYCaFxsmxxrz64b5bV4RAT/mFHCoz+8LbH1cfebCTwv0KCyqBxPZySkwS0aXAn
# DU+3tTbRyV8IpHCj7ArxES5k4MsiK8rxKBMhSVF+BmbTO77665E42FEHypS34lCh
# 8zrTioPLQHsCAwEAAaOCAYswggGHMA4GA1UdDwEB/wQEAwIHgDAMBgNVHRMBAf8E
# AjAAMBYGA1UdJQEB/wQMMAoGCCsGAQUFBwMIMCAGA1UdIAQZMBcwCAYGZ4EMAQQC
# MAsGCWCGSAGG/WwHATAfBgNVHSMEGDAWgBS6FtltTYUvcyl2mi91jGogj57IbzAd
# BgNVHQ4EFgQUpbbvE+fvzdBkodVWqWUxo97V40kwWgYDVR0fBFMwUTBPoE2gS4ZJ
# aHR0cDovL2NybDMuZGlnaWNlcnQuY29tL0RpZ2lDZXJ0VHJ1c3RlZEc0UlNBNDA5
# NlNIQTI1NlRpbWVTdGFtcGluZ0NBLmNybDCBkAYIKwYBBQUHAQEEgYMwgYAwJAYI
# KwYBBQUHMAGGGGh0dHA6Ly9vY3NwLmRpZ2ljZXJ0LmNvbTBYBggrBgEFBQcwAoZM
# aHR0cDovL2NhY2VydHMuZGlnaWNlcnQuY29tL0RpZ2lDZXJ0VHJ1c3RlZEc0UlNB
# NDA5NlNIQTI1NlRpbWVTdGFtcGluZ0NBLmNydDANBgkqhkiG9w0BAQsFAAOCAgEA
# gRrW3qCptZgXvHCNT4o8aJzYJf/LLOTN6l0ikuyMIgKpuM+AqNnn48XtJoKKcS8Y
# 3U623mzX4WCcK+3tPUiOuGu6fF29wmE3aEl3o+uQqhLXJ4Xzjh6S2sJAOJ9dyKAu
# JXglnSoFeoQpmLZXeY/bJlYrsPOnvTcM2Jh2T1a5UsK2nTipgedtQVyMadG5K8TG
# e8+c+njikxp2oml101DkRBK+IA2eqUTQ+OVJdwhaIcW0z5iVGlS6ubzBaRm6zxby
# gzc0brBBJt3eWpdPM43UjXd9dUWhpVgmagNF3tlQtVCMr1a9TMXhRsUo063nQwBw
# 3syYnhmJA+rUkTfvTVLzyWAhxFZH7doRS4wyw4jmWOK22z75X7BC1o/jF5HRqsBV
# 44a/rCcsQdCaM0qoNtS5cpZ+l3k4SF/Kwtw9Mt911jZnWon49qfH5U81PAC9vpwq
# bHkB3NpE5jreODsHXjlY9HxzMVWggBHLFAx+rrz+pOt5Zapo1iLKO+uagjVXKBbL
# afIymrLS2Dq4sUaGa7oX/cR3bBVsrquvczroSUa31X/MtjjA2Owc9bahuEMs305M
# fR5ocMB3CtQC4Fxguyj/OOVSWtasFyIjTvTs0xf7UGv/B3cfcZdEQcm4RtNsMnxY
# L2dHZeUbc7aZ+WssBkbvQR7w8F/g29mtkIBEr4AQQYoxggY2MIIGMgIBATBpMFQx
# CzAJBgNVBAYTAkdCMRgwFgYDVQQKEw9TZWN0aWdvIExpbWl0ZWQxKzApBgNVBAMT
# IlNlY3RpZ28gUHVibGljIENvZGUgU2lnbmluZyBDQSBSMzYCEQDWEF7OjjXPcAi1
# NTmRgr/JMA0GCWCGSAFlAwQCAQUAoHwwEAYKKwYBBAGCNwIBDDECMAAwGQYJKoZI
# hvcNAQkDMQwGCisGAQQBgjcCAQQwHAYKKwYBBAGCNwIBCzEOMAwGCisGAQQBgjcC
# ARUwLwYJKoZIhvcNAQkEMSIEINvr5danbXXHkfrGCjhJ61X+ZOadz0uQEa2m5tms
# eRhCMA0GCSqGSIb3DQEBAQUABIICAHmZKbgMITeaJUyiNfqBVmFkwLJbSFq/ARyY
# Uu1Lcsv4nrvLBG64FvmKUORzHM5D2zhJTwRcfSbjUiIAddPMDEuyB4cc2ewLYBPw
# ChVqLSv91hRSMNkyUr2PHNyuJmxfY9aPqQ+NCTNF0iqW1HvZ08UP4m/Yad9T9MW4
# XZt8GmF2oPGaGHXzF2pSHwfkJYmvwBB44lCesvUZOoX36PuiFnZLRCsfORLlFfcK
# nzFl7uZ/Q7JNXTCQk78dlN207eeVJ0JbWBlu8w6lvekmikeb4kJi/FrD7PpyG/zV
# v+nBRQPIea57HKzIFxOzdFE5+kMhoOv9HN4uBjj7QvFoBVtUaM9kEVPFrYRWP/RQ
# gmeBIsxJUcJVFaQz+s4DOyN96ZXf9Rhzq8j+LV+XuBLNz+3ACY0BplgQPLlabBjW
# nnLHxsltZYkwFFYaGkD/lMadNBO0ZzsLIstYuj6UQtPElWzT1kv5ruBKRocObWHI
# /PdzuzS9jN39jzFEtxXeAj2+X2YoumQgYk+aN6+bqEhEq6U72bGal6QZ4dzmaBAp
# qSxrwH/zOCofkBawCWn+d5FSybCOQh+LPdFSbBEOIPfEY2LX5Hh2oH8X0B00R2ER
# q17KwYbEbY62sLzi4JRNQCHFjlbvuOl2QSLYJOSOr3q9fRspj5z3T+8IJ8y+a7jW
# OiPNb3euoYIDIDCCAxwGCSqGSIb3DQEJBjGCAw0wggMJAgEBMHcwYzELMAkGA1UE
# BhMCVVMxFzAVBgNVBAoTDkRpZ2lDZXJ0LCBJbmMuMTswOQYDVQQDEzJEaWdpQ2Vy
# dCBUcnVzdGVkIEc0IFJTQTQwOTYgU0hBMjU2IFRpbWVTdGFtcGluZyBDQQIQBUSv
# 85SdCDmmv9s/X+VhFjANBglghkgBZQMEAgEFAKBpMBgGCSqGSIb3DQEJAzELBgkq
# hkiG9w0BBwEwHAYJKoZIhvcNAQkFMQ8XDTI0MDIyMDAxNDQyM1owLwYJKoZIhvcN
# AQkEMSIEINrsn97AdESay1bbH/RWRiwVquE+56GNrOVW92Gis58aMA0GCSqGSIb3
# DQEBAQUABIICAJPFmZDrOIWFyzjAt0gqRvJBcTYZ8W9fX6h0+7UGW32cZDlTlHI3
# EI36guCZ2e9eYX8nXiz70mkkHAcY4Xt5UDq0NZQ+kciJtjvZsY7ANqth4GFqdD6M
# M5Aq5H6JCNYESu9Kq0NQ9o76XNibxcX/p2tFkhtAI1Mp+av1zb9+GgnHr4Kj6IVv
# I1VRYewGStJEA1uFpFmbQ+hP1RGi31fm6GtZfJ7QF11kWrbeUDIEXprIIWyWFr1C
# Gpolp70d1xSLkiBcBYQ2baD6BY4FI1gNtYt7vI7Ha6i6z8OOff8fz0zPaJ0fuBxS
# pbIHaeKHd444pVYKqmlvg5WsjmzYq517/0XNWgkviNKDy1n8QRhuJ7rAbh9S2kNo
# A5jlWoXj2neydCz5B8tBen6ImjXTrl6DtZYcmLsMO0pKNec0MovxFdaGKQsHg+g8
# U8Drp6ZLFiZD+QBMuvr+n+rZuAD5ds5eg3AkwODxOgARHjyrhS9aACMr2Barzjin
# bKflrUidBghCEarQyJW9QRvBvZHfsP6iKmPKSaUCQF6XWwRrNd3ICtHioW7qqWaf
# 8ZFt1IE8DmSSm+/jDTZ/I7aXM5Sc8oc6elL3zDeQJgicpldAhy/Qadw3E6ZB7rgZ
# /rK05KfxbAOeA8fTLTxelGLEeelitm/7tqEtGvGk/z8STl5I0a9mEbcI
# SIG # End signature block
