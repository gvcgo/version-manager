package lua_global

import (
	"fmt"
	"testing"
)

func TestInstaller(t *testing.T) {
	script := `
	print("----------installer_config----------")

	ic = vmrNewInstallerConfig()
	ic = vmrAddFlagFiles(ic, "windows", {"windows"})
	ic = vmrAddFlagFiles(ic, "linux", {"linux"})
	ic = vmrAddFlagFiles(ic, "darwin", {"osx"})
	ic = vmrAddBinaryDirs(ic, "windows", {"windows", "bin"})
	ic = vmrAddBinaryDirs(ic, "linux", {"linux", "bin64"})
	ic = vmrAddBinaryDirs(ic, "darwin", {"osx", "bin"})
	ic = vmrAddAdditionalEnvs(ic, "PATH", {"xxx", "bin"}, ">=1.0.0")
	ic = vmrEnableFlagDirExcepted(ic)

	print(ic)
	`

	if l, err := ExecuteLuaScriptL(script); err != nil {
		if l != nil {
			l.Close()
		}
		t.Error(err)
	} else {
		ic := GetInstallerConfig(l)
		// fmt.Println("isntaller_config: ", ic)
		fmt.Println("installer_config_flagFiles: ", ic.FlagFiles)
		fmt.Println("installer_config_flagDirExcepted: ", ic.FlagDirExcepted)
		fmt.Println("installer_config_binaryDirs: ", ic.BinaryDirs)
		fmt.Println("installer_config_additionalEnvs: ", ic.AdditionalEnvs)
	}
}
