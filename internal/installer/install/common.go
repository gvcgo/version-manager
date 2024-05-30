package install

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/gvcgo/version-manager/internal/cnf"
)

const (
	VerisonDirPattern        string = "%s_versions"
	VersionInstallDirPattern string = "%s-%s"
)

func GetSDKVersionDir(sdkName string) string {
	versionDir := cnf.GetVersionsDir()
	d := filepath.Join(versionDir, fmt.Sprintf(VerisonDirPattern, sdkName))
	os.MkdirAll(d, os.ModePerm)
	return d
}
