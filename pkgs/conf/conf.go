package conf

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/pkgs/utils"
)

func init() {
	LoadConfigFile()
}

/*
ENVs
1. Proxies for Version Manager.
2. App installation directories.
3. Multi-thread downloading.

Examples:
export VM_PROXY_URI="http://127.0.0.1:2023"
export VM_REVERSE_PROXY_URL="https://gvc.1710717.xyz/proxy/"
export VM_APP_INSTALL_DIR="~/.vm/"
*/
const (
	VMProxyEnvName                string = "VM_PROXY_URI"
	VMReverseProxyEnvName         string = "VM_REVERSE_PROXY_URL"
	VMWorkDirEnvName              string = "VM_APP_INSTALL_DIR"
	VMDownloadThreadsEnvName      string = "VM_DOWNLOAD_THREADS"
	VMUseMirrorInChinaEnvName     string = "VM_USE_MIRROR_IN_CHINA"
	VMOnlyInCurrentSessionEnvName string = "VM_ONLY_IN_CURRENT_SESSION" // uses a version only in current session.
	VMLockedVersionEnvName        string = "VM_LOCKED_VERSIONS"
)

type Config struct {
	ProxyURI           string `json:"proxy_uri"`
	ReverseProxy       string `json:"reverse_proxy"`
	AppInstallationDir string `json:"app_installation_dir"`
}

func GetManagerDir() string {
	homeDir, _ := os.UserHomeDir()
	managerDir := filepath.Join(homeDir, ".vm")
	os.MkdirAll(managerDir, os.ModePerm) // create dir if not exist.
	return managerDir
}

func GetConfPath() string {
	managerDir := GetManagerDir()
	return filepath.Join(managerDir, "config.json")
}

func LoadConfigFile() (c *Config) {
	c = &Config{}
	cfgPath := GetConfPath()

	if ok, _ := gutils.PathIsExist(cfgPath); ok {
		data, _ := os.ReadFile(cfgPath)
		json.Unmarshal(data, c)
	} else {
		return
	}
	// set ENVs.
	if c.ProxyURI != "" {
		os.Setenv(VMProxyEnvName, c.ProxyURI)
	}
	if c.ReverseProxy != "" {
		os.Setenv(VMReverseProxyEnvName, c.ReverseProxy)
	}
	if c.AppInstallationDir != "" {
		os.Setenv(VMWorkDirEnvName, c.AppInstallationDir)
	}
	return
}

func SaveConfigFile(c *Config) {
	cfgPath := GetConfPath()
	oldCfg := &Config{}
	if data, err := os.ReadFile(cfgPath); err == nil {
		json.Unmarshal(data, oldCfg)
	}

	if c.ProxyURI != "" {
		oldCfg.ProxyURI = c.ProxyURI
	}
	if c.ReverseProxy != "" {
		oldCfg.ReverseProxy = c.ReverseProxy
	}
	if c.AppInstallationDir != "" {
		oldCfg.AppInstallationDir = c.AppInstallationDir
	}
	if content, err := json.MarshalIndent(oldCfg, "", "    "); err == nil {
		os.WriteFile(cfgPath, content, 0o644)
	}
}

/*
======================================
get value from ENVs.
======================================
*/

// Use mirror site in China.
func UseMirrorSiteInChina() bool {
	return gconv.Bool(os.Getenv(VMUseMirrorInChinaEnvName))
}

// Sets proxy for fetcher.
func GetFetcher() *request.Fetcher {
	dthreads := os.Getenv(VMDownloadThreadsEnvName)
	num := gconv.Int(dthreads)
	if num <= 0 {
		num = 1
	}
	r := request.NewFetcher()
	r.SetThreadNum(num)
	r.Proxy = os.Getenv(VMProxyEnvName)
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

// Apps installation dir.
func GetVersionManagerWorkDir() string {
	d := os.Getenv(VMWorkDirEnvName)
	if d == "" {
		d = GetManagerDir()
	}
	os.MkdirAll(d, os.ModePerm)
	return d
}

// Binaries dir.
func GetAppBinDir() string {
	d := filepath.Join(GetVersionManagerWorkDir(), "bin")
	os.MkdirAll(d, os.ModePerm)
	return d
}

// ZipFile dir.
func GetZipFileDir(appName string) string {
	d := filepath.Join(GetVersionManagerWorkDir(), "cache", appName)
	os.MkdirAll(d, os.ModePerm)
	return d
}

// Temp dir.
func GetVMTempDir() string {
	d := filepath.Join(GetVersionManagerWorkDir(), "tmp")
	os.MkdirAll(d, os.ModePerm)
	return d
}

// versions dir.
func GetVMVersionsDir(appName string) string {
	dirName := fmt.Sprintf("%s_versions", appName)
	versionsDir := filepath.Join(GetVersionManagerWorkDir(), "versions")
	utils.ClearEmptyDirs(versionsDir) // remove empty dirs left behind.
	d := filepath.Join(versionsDir, dirName)
	os.MkdirAll(d, os.ModePerm)
	return d
}
