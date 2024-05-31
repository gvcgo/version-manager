package post

import (
	"fmt"
	"path/filepath"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/download"
)

func init() {
	RegisterPostInstallHandler(UPXSdkName, PostInstallForUPX)
}

const (
	UPXSdkName string = "upx"
)

/*
post-installation handler for UPX.
*/
func PostInstallForUPX(versionName string, version download.Item) {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf("%s_versions", UPXSdkName))
	upxInstallDir := filepath.Join(d, fmt.Sprintf("%s-%s", UPXSdkName, versionName))
	if runtime.GOOS == gutils.Windows {
		binPath := filepath.Join(upxInstallDir, "upx")
		gutils.ExecuteSysCommand(true, upxInstallDir, "chmod", "+x", binPath)
	}
}
