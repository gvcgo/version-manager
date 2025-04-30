package post

import (
	"fmt"
	"path/filepath"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global"
)

func init() {
	RegisterPostInstallHandler(ZigSdkName, PostInstallForZig)
}

const (
	ZigSdkName string = "zig"
)

/*
post-installation handler for Zig.
*/
func PostInstallForZig(versionName string, version lua_global.Item) {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf("%s_versions", ZigSdkName))
	zigInstallDir := filepath.Join(d, fmt.Sprintf("%s-%s", ZigSdkName, versionName))
	if runtime.GOOS != gutils.Windows {
		binPath := filepath.Join(zigInstallDir, "zig")
		gutils.ExecuteSysCommand(true, zigInstallDir, "chmod", "+x", binPath)
	}
}
