package self

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/input"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/shell"
)

/*
Install vmr itself.
*/
func InstallSelf() {
	// Handle old versions.
	DetectAndRemoveOldVersions()

	/*
		------------------
		install vmr.
		------------------
	*/
	excutablePath, _ := os.Executable()
	vmrWorkDir := cnf.GetVMRWorkDir()

	// vmr is already installed.
	if strings.HasPrefix(excutablePath, vmrWorkDir) {
		return
	}
	binName := filepath.Base(excutablePath)
	installPath := filepath.Join(vmrWorkDir, binName)

	os.RemoveAll(installPath)
	err := gutils.CopyAFile(excutablePath, installPath)
	if err != nil {
		gprint.PrintError("install vmr failed: %+v", err)
	}

	sh := shell.NewShell()
	sh.WriteVMEnvToShell()

	// Generate update script.
	SetUpdateScript()

	// Set your sdk installation directory.
	fmt.Println(gprint.YellowStr(`Please set the sdk installation directory for VMR.`))
	fmt.Println(gprint.YellowStr("The sdk installation directory is used to store the SDKs Installed by VMR."))
	fmt.Println(gprint.YellowStr("If you left it as blank, the sdk installation directory will be '$HOME/.vmr/'."))
	fmt.Println("")
	ipt := input.NewInput(input.WithPlaceholder("$HOME/.vm/"), input.WithPrompt("SDK Installation Dir: "))
	ipt.Run()
	appDir := ipt.Value()
	if appDir == "" {
		// use default value.
		appDir = cnf.GetVMRWorkDir()
	}
	cnf.DefaultConfig.SDKIntallationDir = appDir
	cnf.DefaultConfig.Save()
}
