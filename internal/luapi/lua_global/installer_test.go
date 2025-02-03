package lua_global

import (
	"fmt"
	"testing"
)

var installerScript = `
print("----------installer_config----------")

ic = newInstallerConfig()
ic = addFlagFiles(ic, "windows", {"windows"})
ic = addFlagFiles(ic, "linux", {"linux"})
ic = addFlagFiles(ic, "darwin", {"osx"})
ic = addBinaryDirs(ic, "windows", {"windows", "bin"})
ic = addBinaryDirs(ic, "linux", {"linux", "bin64"})
ic = addBinaryDirs(ic, "darwin", {"osx", "bin"})
ic = addAdditionalEnvs(ic, "PATH", {"xxx", "bin"}, ">=1.0.0")
ic = enableFlagDirExcepted(ic)

print(ic)
`

func TestInstaller(t *testing.T) {
	ll := NewLua()
	defer ll.Close()
	L := ll.GetLState()

	if err := L.DoString(installerScript); err != nil {
		t.Error(err)
	}

	ic := GetInstallerConfig(L)
	// fmt.Println("isntaller_config: ", ic)
	fmt.Println("installer_config_flagFiles: ", ic.FlagFiles)
	fmt.Println("installer_config_flagDirExcepted: ", ic.FlagDirExcepted)
	fmt.Println("installer_config_binaryDirs: ", ic.BinaryDirs)
	fmt.Println("installer_config_additionalEnvs: ", ic.AdditionalEnvs)
}
