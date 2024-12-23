package self

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
)

/*
Set update script.
*/
const (
	updateScriptName string = "vmr-update"
)

var WinScript string = `cd "$env:USERPROFILE"
powershell -c "irm https://scripts.vmr.us.kg/windows | iex"`

var UnixScript string = `#!/bin/sh
cd ~
curl --proto '=https' --tlsv1.2 -sSf https://scripts.vmr.us.kg | sh`

func setUpdateForWindows() {
	scriptPath := filepath.Join(cnf.GetVMRWorkDir(), fmt.Sprintf("%s.bat", updateScriptName))
	os.WriteFile(scriptPath, []byte(WinScript), os.ModePerm)
}

func setUpdateForUnix() {
	scriptPath := filepath.Join(cnf.GetVMRWorkDir(), updateScriptName)
	os.WriteFile(scriptPath, []byte(UnixScript), os.ModePerm)
}

func SetUpdateScript() {
	if runtime.GOOS == gutils.Windows {
		setUpdateForWindows()
	} else {
		setUpdateForUnix()
	}
}
