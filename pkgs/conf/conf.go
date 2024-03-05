package conf

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
)

/*
Proxies for Version Manager.

Examples:
export VM_PROXY_URI="http://127.0.0.1:2023"
export VM_REVERSE_PROXY_URL="https://gvc.1710717.xyz/proxy/"
*/
const (
	VMProxyEnvName        string = "VM_PROXY_URI"
	VMReverseProxyEnvName string = "VM_REVERSE_PROXY_URL"
)

// Sets proxy for fetcher.
func GetFetcher() *request.Fetcher {
	r := request.NewFetcher()
	r.SetProxyEnvName(VMProxyEnvName)
	return r
}

// Decorate url with reverse proxy.
func DecorateUrl(dUrl string) string {
	rp := os.Getenv(VMReverseProxyEnvName)
	if gutils.VerifyUrls(rp) {
		return fmt.Sprintf("%s/%s", strings.TrimRight(rp, "/"), dUrl)
	}
	return dUrl
}

/*
Installation directories.
*/
const (
	VMWorkDirEnvName string = "VM_APP_INSTALL_DIR"
)

// Apps installation dir.
func GetVersionManagerWorkDir() string {
	d := os.Getenv(VMWorkDirEnvName)
	if d == "" {
		homeDir, _ := os.UserHomeDir()
		d = filepath.Join(homeDir, ".vm")
	}
	return d
}

// Binaries dir.
func GetAppBinDir() string {
	return filepath.Join(GetVersionManagerWorkDir(), "bin")
}

// ZipFile dir.
func GetZipFileDir() string {
	return filepath.Join(GetVersionManagerWorkDir(), "cache")
}

// Temp dir.
func GetVMTempDir() string {
	return filepath.Join(GetVersionManagerWorkDir(), "tmp")
}

// versions dir.
func GetVMVersionsDir(appName string) string {
	dirName := fmt.Sprintf("%s_versions", appName)
	return filepath.Join(GetVersionManagerWorkDir(), dirName)
}
