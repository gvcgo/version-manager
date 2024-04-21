package self

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gtea/input"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/envs"
	"github.com/gvcgo/version-manager/pkgs/conf"
)

/*
Installs vmr itself.
*/
func InstallVmr() {
	vmBinName := "vmr"
	oldBinName := "vm"
	if runtime.GOOS == gutils.Windows {
		vmBinName = "vmr.exe"
		oldBinName = "vm.exe"
	}
	binPath := filepath.Join(conf.GetManagerDir(), vmBinName)
	oldBinPath := filepath.Join(conf.GetManagerDir(), oldBinName)
	os.RemoveAll(oldBinPath)

	currentBinPath, _ := os.Executable()
	currentDir := filepath.Dir(currentBinPath)

	if currentDir == conf.GetManagerDir() {
		gprint.PrintWarning("vmr is already installed, please do not repeat the installation.")
		os.Exit(0)
	}

	// If there is an old vmr, and the current one is not in $HOME/.vmr, then delete the old one first.
	if ok, _ := gutils.PathIsExist(binPath); ok {
		os.RemoveAll(binPath)
	}

	if strings.HasSuffix(currentBinPath, vmBinName) {
		gutils.CopyFile(currentBinPath, binPath)
	}
	em := envs.NewEnvManager()
	defer em.CloseKey()
	em.AddToPath(conf.GetManagerDir())

	if ok, _ := gutils.PathIsExist(conf.GetConfPath()); ok {
		return
	}
	// Sets app installation Dir.
	fmt.Println(gprint.CyanStr(`Enter the SDK installation directory for vmr:`))
	fmt.Println("")
	ipt := input.NewInput(input.WithPlaceholder("$HOME/.vm/"), input.WithPrompt("SDK Installation Dir: "))
	ipt.Run()
	appDir := ipt.Value()
	if appDir == "" {
		appDir = conf.GetManagerDir()
	}
	conf.SaveConfigFile(&conf.Config{AppInstallationDir: appDir})
}
