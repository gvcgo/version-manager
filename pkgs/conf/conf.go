package conf

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
)

func init() {
	LoadConfigFile()
}

/*
ENVs
1. Proxies for Version Manager.
2. App installation directories.

Examples:
export VM_PROXY_URI="http://127.0.0.1:2023"
export VM_REVERSE_PROXY_URL="https://gvc.1710717.xyz/proxy/"
export VM_APP_INSTALL_DIR="~/.vm/"
*/
const (
	VMProxyEnvName        string = "VM_PROXY_URI"
	VMReverseProxyEnvName string = "VM_REVERSE_PROXY_URL"
	VMWorkDirEnvName      string = "VM_APP_INSTALL_DIR"
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

func getConfPath() string {
	managerDir := GetManagerDir()
	return filepath.Join(managerDir, "config.json")
}

func LoadConfigFile() (c *Config) {
	c = &Config{}
	cfgPath := getConfPath()
	if ok, _ := gutils.PathIsExist(cfgPath); ok {
		data, _ := os.ReadFile(cfgPath)
		json.Unmarshal(data, c)
	} else {
		return
	}
	// set ENVs.
	if c.ProxyURI == "" {
		os.Setenv(VMProxyEnvName, c.ProxyURI)
	}
	if c.ReverseProxy == "" {
		os.Setenv(VMReverseProxyEnvName, c.ReverseProxy)
	}
	if c.AppInstallationDir == "" {
		os.Setenv(VMWorkDirEnvName, c.AppInstallationDir)
	}
	return
}

func SaveConfigFile(c *Config) {
	cfgPath := getConfPath()
	oldCfg := &Config{}
	if data, err := os.ReadFile(cfgPath); err == nil {
		json.Unmarshal(data, oldCfg)
	}

	if c.ProxyURI != "" {
		oldCfg.ProxyURI = c.ProxyURI
	}
	if c.ProxyURI != "" {
		oldCfg.ReverseProxy = c.ReverseProxy
	}
	if c.AppInstallationDir != "" {
		oldCfg.AppInstallationDir = c.AppInstallationDir
	}
	if content, err := json.Marshal(oldCfg); err == nil {
		os.WriteFile(cfgPath, content, os.ModePerm)
	}
}

/*
======================================
get value from ENVs.
======================================
*/

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
	d := filepath.Join(GetVersionManagerWorkDir(), "versions", dirName)
	os.MkdirAll(d, os.ModePerm)
	return d
}
