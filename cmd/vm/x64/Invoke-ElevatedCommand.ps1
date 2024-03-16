<#
.SYNOPSIS
Executes a ScriptBlock or script file in a new elevated instance of PowerShell using gsudo.

.DESCRIPTION
The Invoke-ElevatedCommand cmdlet serializes a ScriptBlock or script file and executes it in an elevated PowerShell instance using gsudo. The elevated command runs in a separate process, which means it cannot directly access or modify variables from the invoking scope. 

The cmdlet supports passing arguments to the ScriptBlock or script file using the -ArgumentList parameter, which can be accessed with the $args automatic variable within the ScriptBlock or script file. Additionally, you can provide input from the pipeline using $Input, which will be serialized and made available within the elevated execution context.

The result of the elevated command is serialized, sent back to the non-elevated instance, deserialized, and returned. This means object graphs are recreated as PSObjects.

Optionally, you can check for "$LastExitCode -eq 999" to determine if gsudo failed to elevate, such as when the UAC popup is cancelled.

.PARAMETER ScriptBlock
Specifies a ScriptBlock to be executed in an elevated PowerShell instance. For example: { Get-Process Notepad }

.PARAMETER FilePath
Specifies the path to a script file to be executed in an elevated PowerShell instance.

.PARAMETER ArgumentList
Provides a list of elements that will be accessible inside the ScriptBlock or script as $args[0], $args[1], and so on.

.PARAMETER LoadProfile
Loads the user profile in the elevated PowerShell instance, regardless of the gsudo configuration setting PowerShellLoadProfile.

.PARAMETER NoProfile
Does not load the user profile in the elevated PowerShell instance.

.PARAMETER RunAsTrustedInstaller
Runs the command as the TrustedInstaller user.

.PARAMETER RunAsSystem
Runs the command as the SYSTEM user.

.PARAMETER ClearCache
Clears the gsudo cache before executing the command.

.PARAMETER NewWindow
Opens a new window for the elevated command.

.PARAMETER KeepNewWindowOpen
Keeps the new window open after the elevated command finishes execution.

.PARAMETER KeepShell
Keeps the shell open after the elevated command finishes execution.

.PARAMETER NoKeep
Closes the shell after the elevated command finishes execution.

.PARAMETER InputObject
You can pipe any object to this function. The object will be serialized and available as $Input within the ScriptBlock or script.

.INPUTS
System.Object

.OUTPUTS
The output of the ScriptBlock or script executed in the elevated context.

.EXAMPLE
Invoke-ElevatedCommand { return Get-Content 'C:\My Secret Folder\My Secret.txt' }

.EXAMPLE
Get-Process notepad | Invoke-ElevatedCommand { $input | Stop-Process }

.EXAMPLE
$a = 1; $b = Invoke-ElevatedCommand { $args[0] + 10 } -ArgumentList $a; Write-Host "Sum returned: $b"
Sum returned: 11

.EXAMPLE 
Invoke-ElevatedCommand { Get-Process explorer } | ForEach-Object { $_.Id }

.LINK
https://github.com/gerardog/gsudo
#>
[CmdletBinding(DefaultParameterSetName='ScriptBlock')]
param
(
    # The script block to execute in an elevated context.
    [Parameter(Mandatory = $true, Position = 0, ParameterSetName='ScriptBlock')] [System.Management.Automation.ScriptBlock]
[ArgumentCompleter( { param ()
			# Auto complete with last 5 ran commands.
			$lastCommands = Get-History | Select-Object -last 5 | % { "{ $($_.CommandLine) }" }
		
			if ($lastCommands -is [System.Array]) {
				# Last one first.
				$lastCommands[($lastCommands.Length - 1)..0] | % { $_ };
			}
			elseif ($lastCommands) {
				# Only one command.
				$lastCommands;
			}
        } )]

		$ScriptBlock,

	# Alternarive file to be executed in an elevated PowerShell instance.
    [Parameter(Mandatory = $true, ParameterSetName='ScriptFile')] [String] $FilePath,
	
    [Parameter(Mandatory = $false)] [System.Object[]] $ArgumentList,
	
	[Parameter(ParameterSetName='ScriptBlock')] [switch] $LoadProfile,
	[Parameter(ParameterSetName='ScriptBlock')] [switch] $NoProfile,
	
	[Parameter()] [switch] $RunAsTrustedInstaller,
	[Parameter()] [switch] $RunAsSystem,
	[Parameter()] [switch] $ClearCache,
	
	[Parameter()] [switch] $NewWindow,
	[Parameter()] [switch] $KeepNewWindowOpen,
	[Parameter()] [switch] $KeepShell,
	[Parameter()] [switch] $NoKeep,
	
	[ValidateSet('Low', 'Medium', 'MediumPlus', 'High', 'System')]
	[System.String]$Integrity,

	[Parameter()] [System.Management.Automation.PSCredential] $Credential,
    [Parameter(ValueFromPipeline)] [pscustomobject] $InputObject
)
Begin {
	$inputArray = @() 
}
Process {
	foreach ($item in $InputObject) {
		# Add the modified item to the output array
		$inputArray += $item
	}
}
End {
	$gsudoArgs = @()

	if ($PSCmdlet.MyInvocation.BoundParameters["Debug"].IsPresent) { $gsudoArgs += '--debug' }

	if ($LoadProfile)	{ $gsudoArgs += '--LoadProfile' }
	if ($RunAsTrustedInstaller)	{ $gsudoArgs += '--ti' }
	if ($RunAsSystem)	{ $gsudoArgs += '-s' }
	if ($ClearCache)	{ $gsudoArgs += '-k' }
	if ($NewWindow)		{ $gsudoArgs += '-n' }
	if ($KeepNewWindowOpen)		{ $gsudoArgs += '--KeepWindow' }
	if ($NoKeep)		{ $gsudoArgs += '--close' }
	if ($Integrity)	{ $gsudoArgs += '--integrity'; $gsudoArgs += $Integrity}

	if ($Credential) {
		$CurrentSid = ([System.Security.Principal.WindowsIdentity]::GetCurrent()).User.Value;
		$gsudoArgs += "-u", $credential.UserName
		
		# At the time of writing this, there is no way (considered secure) to send the password to gsudo. So instead of sending the password, lets start a credentials cache instance.	
		$p = Start-Process "gsudo.exe" -Args "-u $($credential.UserName) gsudoservice $PID $CurrentSid All 00:05:00" -credential $Credential -LoadUserProfile -WorkingDirectory "$env:windir" -WindowStyle Hidden -PassThru 
		$p.WaitForExit();
		Start-Sleep -Seconds 1
	} 

	if ($PSVersionTable.PSVersion.Major -le 5) {
		$pwsh = "powershell.exe" 
	} else 	{
		$pwsh = "pwsh.exe" 
	}

	if ($ScriptBlock) {	
		if ($NoProfile) { 
			$gsudoArgs += '-d';
			$gsudoArgs += $pwsh;
			$gsudoArgs += '-NoProfile';
			$gsudoArgs += '-NoLogo';
			
			if ($KeepShell)		{ $gsudoArgs += '--NoExit' }
		} else {
			if ($KeepShell)		{ $gsudoArgs += '--KeepShell' }
		}

		if ($myInvocation.expectingInput) {
			$inputArray | gsudo.exe @gsudoArgs $ScriptBlock -args $ArgumentList
		} else {
			gsudo.exe @gsudoArgs $ScriptBlock -args $ArgumentList
		}
	} else {
		if ($myInvocation.expectingInput) {
			$inputArray | gsudo.exe @gsudoArgs -args $ArgumentList
		} else {
			gsudo.exe @gsudoArgs -d $pwsh -File $FilePath -args $ArgumentList
		}
	}
}

# SIG # Begin signature block
# MIIr1gYJKoZIhvcNAQcCoIIrxzCCK8MCAQExDzANBglghkgBZQMEAgEFADB5Bgor
# BgEEAYI3AgEEoGswaTA0BgorBgEEAYI3AgEeMCYCAwEAAAQQH8w7YFlLCE63JNLG
# KX7zUQIBAAIBAAIBAAIBAAIBADAxMA0GCWCGSAFlAwQCAQUABCDBsMKKwy6l1Yn1
# CK3GA6y33F5tnbaIvKMLdSooFC2BzaCCJPYwggVvMIIEV6ADAgECAhBI/JO0YFWU
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
# ARUwLwYJKoZIhvcNAQkEMSIEIAEcVzlNXBsfuYalIHkLAuxAWAbFsgkCL61reVr2
# USnXMA0GCSqGSIb3DQEBAQUABIICAFzHelMKMszp1iXRzDdAiHxlwtJ9zQhTK6QJ
# BZCcptNKJt1mJ1HiB5CnDl0pTmK2o62UXwyHoAW4S1+3IcKqpMi1MYektM2fgAry
# 6R3C2KBCfhrTp9BHFxRjLcAFDhRTbPlJ0TxuIBaFKQeH4tUU+7PA5nevqg7TUjr/
# BBxHxSktQT3qQnqZxj1qfPLkNsWJSe1F/nxXf05SsMgrsgoeUxebP5vgj+Wn2CMW
# 4C9SfFms5IE4NVF+ggruXwVZvMH0u8TDC/qVTOsLw7Vx/FqkbuxukVp609aBtwL5
# iv+bPw0ZHlk7B2L+pjDckj17/+HOQ+ejiug2mjjLo5SblqZgJD4zBn3jnjFYjRCB
# OhSYxGLH6PH6IXpVzsPD4aeGpbnFS9wDexcRZyoHuVxN7PnebqrcO2GykwA3X10h
# xt7/CfvSUvsOiMTBKX1zatbYKWu6sCn9W6t2fndMx2Fh3h1SRp7wy+jpsNe/C5FP
# JBf7R0UOsRyoqO986NBDqCx6ygwR7fH3KJ6RX8FDyCG0r68jkCP+GGtL10WSMXMW
# nstAZIWjRcxZu6JH131d2xj8wiS2/0D8gbpwhc0lg0Gy9WHpufR6rqmgr2ngUYVP
# Ru3krbf1axsqM3cuEQcnhcO0Bngt61zYQEi7aQvlCZeWkSsOyZZ3rAWaRAh7cF6J
# qhUdFH6QoYIDIDCCAxwGCSqGSIb3DQEJBjGCAw0wggMJAgEBMHcwYzELMAkGA1UE
# BhMCVVMxFzAVBgNVBAoTDkRpZ2lDZXJ0LCBJbmMuMTswOQYDVQQDEzJEaWdpQ2Vy
# dCBUcnVzdGVkIEc0IFJTQTQwOTYgU0hBMjU2IFRpbWVTdGFtcGluZyBDQQIQBUSv
# 85SdCDmmv9s/X+VhFjANBglghkgBZQMEAgEFAKBpMBgGCSqGSIb3DQEJAzELBgkq
# hkiG9w0BBwEwHAYJKoZIhvcNAQkFMQ8XDTI0MDIyMDAxNDQyM1owLwYJKoZIhvcN
# AQkEMSIEIGcC112JX/5EHETTl156G/Hn7v53/UhhFa5AYD89qa1wMA0GCSqGSIb3
# DQEBAQUABIICABTs7osTDo5nMUNID+Wb7H1KvAt+rdc2WjlslUcTMo797xW5Kp/I
# 2uU27HWuwUsNeLCQLffL7RXkZ2cSXPy0fwv06Qy9mytErmFm9myBN2qKNg16OB8a
# YKnTQqZ0tswwdJ+ual4L024qC5WGSqRfVVKy0Tdv7JshwmcR2mCqkj3I5/qRjW6j
# aGiBDOnYr8yAnSztYwAOEiWCwOhcnEKq5310x1oLaqzvdtSHvhOEiuEo1Cho/Wgu
# FhNsADYaxIRQT4zSPSnOGY2Pf+4SpUGF73dshX+1XoXDRbKsGqwUyAOkCLD6J3zH
# NTdHe+U7CpeEIwVtzoV1rgaGM9TEf7UvYpLtDqxtBelPdcciOJJbBE5rKGYrMHsh
# IvBNj98G76IX4mRlalxh/bwI43BRrROoXYAOTbjPx9HxSaP81vWcndnBKhnep0gO
# lYTXRz9dbplDR5R/DohFP4s3CMT6epE8UdKffy8FkeGNcIHCG4OogUdb0twzK1CA
# N0iv4SqktxafNEMQ6Px5puaVZMdoOzhHdU7sD3SKl98ytcQU+IfV76lAnd1Zia6g
# z1ZON+IMav4uM6F6aJfR2AaBnkeCH7aA+HQLABvRUQkUnAOVO0s2WAoNhMzuIWOG
# txlm7nKEYOe/TPnSmaDLZCfrfLkW0GslzX+wCJoxpNrO7CF8nW0PiJJf
# SIG # End signature block
