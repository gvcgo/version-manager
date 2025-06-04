package self

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
Set update script.
*/
const (
	updateScriptName string = "vmr-update"
)

var (
	WinS string = `cd %HOMEPATH%
powershell -c "irm https://scripts.%s/windows | iex"`

	WinScript string = fmt.Sprintf(WinS, cnf.DefaultDomain)
)

var WinMingwScript string = `#!/bin/sh
cd ~
powershell %s`

var (
	UnixS string = `#!/bin/sh
cd ~
curl --proto '=https' --tlsv1.2 -sSf https://scripts.%s | sh`

	UnixScript string = fmt.Sprintf(UnixS, cnf.DefaultDomain)
)

func setUpdateForWindows() {
	scriptPath := filepath.Join(cnf.GetVMRWorkDir(), fmt.Sprintf("%s.bat", updateScriptName))
	os.WriteFile(scriptPath, []byte(WinScript), os.ModePerm)

	mingwScriptPath := filepath.Join(cnf.GetVMRWorkDir(), fmt.Sprintf("%s.sh", updateScriptName))
	batPath := utils.ConvertWindowsPathToMingwPath(scriptPath)
	os.WriteFile(mingwScriptPath, []byte(fmt.Sprintf(WinMingwScript, batPath)), os.ModePerm)
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
