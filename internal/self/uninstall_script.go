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
set uninstall script.
*/
const (
	unInstallScriptName string = `vmr-uninstall`
)

var UnixRemoveScript string = `#!/bin/sh
vmr Uins
rm -rf %s`

var WinRemoveScript string = `vmr Uins
rmdir /s /q %s`

func SetUninstallScript() {
	script := UnixRemoveScript
	scriptName := unInstallScriptName
	if runtime.GOOS == gutils.Windows {
		script = WinRemoveScript
		scriptName = unInstallScriptName + ".bat"
	}
	script = fmt.Sprintf(script, cnf.GetVMRWorkDir())

	scriptPath := filepath.Join(cnf.GetVMRWorkDir(), scriptName)
	os.WriteFile(scriptPath, []byte(script), os.ModePerm)

	if runtime.GOOS != gutils.Windows {
		gutils.ExecuteSysCommand(true, "", "chmod", "+x", scriptPath)
	}
}
