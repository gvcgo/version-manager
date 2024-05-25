package cnf

import (
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strings"

	toml "github.com/pelletier/go-toml/v2"
)

var DefaultConfig *VMRConf

func init() {
	DefaultConfig = NewVMRConf()
}

const (
	DefaultReverseProxy       string = "https://gvc.1710717.xyz/proxy/"
	DefaultHostUrl            string = "https://raw.githubusercontent.com/gvcgo/vsources/main"
	DefaultInstallationHost   string = "" // sdk installation config file
	SDKNameListFileUrl        string = `/sdk-list.version.json`
	VersionFileUrlPattern     string = `/%s.version.json`
	SDKInstallationUrlPattern string = `/%s.install.toml`
	VMRWorkDirName            string = ".vmr"
)

/*
Envs
*/
const (
	VMRSdkInstallationDirEnv string = "VMR_SDK_INSTALLATION_DIR"
	VMRHostUrlEnv            string = "VMR_HOST"
	VMRInstallationHostEnv   string = "VMR_INSTALLATION_HOST"
	VMRReverseProxyEnv       string = "VMR_REVERSE_PROXY"
)

func GetVMRWorkDir() string {
	homeDir, _ := os.UserHomeDir()
	p := filepath.Join(homeDir, VMRWorkDirName)
	os.MkdirAll(p, os.ModePerm)
	return p
}

func GetVMRConfFilePath() string {
	return filepath.Join(GetVMRWorkDir(), "conf.toml")
}

func GetCacheDir() string {
	installationDir := filepath.Dir(GetVersionsDir())
	p := filepath.Join(installationDir, "cache")
	os.MkdirAll(p, os.ModePerm)
	return p
}

func GetVersionsDir() string {
	sdkInstallationDir := os.Getenv(VMRSdkInstallationDirEnv)
	if sdkInstallationDir != "" {
		// Use customed directory.
		vp := filepath.Join(sdkInstallationDir, "versions")
		os.MkdirAll(vp, os.ModePerm)
		return vp
	} else {
		/*
			Use ~/.vmr/versions by default.
		*/
		vp := filepath.Join(GetVMRWorkDir(), "versions")
		os.MkdirAll(vp, os.ModePerm)
		return vp
	}
}

/*
Temp directory is for unarchiving sdk files.
And will be removed after the temp files are copied to installation directory.
*/
func GetTempDir() string {
	tDir := filepath.Join(GetVMRWorkDir(), "temp")
	os.MkdirAll(tDir, os.ModePerm)
	return tDir
}

/*
This directory is for storing sdk installation config files.
*/
func GetSDKInstallationConfDir() string {
	icd := filepath.Join(GetVMRWorkDir(), "install_confs")
	os.MkdirAll(icd, os.ModePerm)
	return icd
}

type VMRConf struct {
	ProxyUri            string `json,toml:"proxy_uri"`
	ReverseProxy        string `json,toml:"reverse_proxy"`
	SDKIntallationDir   string `json,toml:"sdk_installation_dir"`
	VersionHostUrl      string `json,toml:"version_host_url"`
	InstallationHostUrl string `json,toml:"installation_host_url"`
}

func NewVMRConf() (v *VMRConf) {
	v = &VMRConf{}
	v.Load()
	if v.SDKIntallationDir != "" {
		os.Setenv(VMRSdkInstallationDirEnv, v.SDKIntallationDir)
	}
	if v.VersionHostUrl != "" {
		os.Setenv(VMRHostUrlEnv, v.VersionHostUrl)
	}
	if v.InstallationHostUrl != "" {
		os.Setenv(VMRInstallationHostEnv, v.InstallationHostUrl)
	}
	return v
}

func (v *VMRConf) Load() {
	path := GetVMRConfFilePath()
	content, _ := os.ReadFile(path)
	if len(content) > 0 {
		toml.Unmarshal(content, v)
	}
}

func (v *VMRConf) Save() {
	path := GetVMRConfFilePath()
	content, _ := toml.Marshal(v)
	os.WriteFile(path, content, os.ModePerm)
}

func (v *VMRConf) SetProxyUri(sUri string) {
	if sUri == "" {
		return
	}
	v.Load()
	v.ProxyUri = sUri
	v.Save()
}

func (v *VMRConf) SetReverseProxy(sUri string) {
	if sUri == "" {
		return
	}
	v.Load()
	v.ReverseProxy = sUri
	v.Save()
}

// reverse proxy
func GetReverseProxyUri() string {
	rp := os.Getenv(VMRReverseProxyEnv)
	if rp == "" {
		rp = DefaultReverseProxy
	}
	if !strings.HasSuffix(rp, "/") {
		rp = rp + "/"
	}
	return rp
}

// sdk-list.version.json
func GetSDKListFileUrl() string {
	host := os.Getenv(VMRHostUrlEnv)
	if host == "" {
		host = DefaultHostUrl
	}
	u, _ := url.JoinPath(host, SDKNameListFileUrl)
	u = GetReverseProxyUri() + u
	return u
}

// {sdkname}.version.json file
func GetVersionFileUrlBySDKName(sdkName string) string {
	host := os.Getenv(VMRHostUrlEnv)
	if host == "" {
		host = DefaultHostUrl
	}
	u, _ := url.JoinPath(host, fmt.Sprintf(VersionFileUrlPattern, sdkName))
	u = GetReverseProxyUri() + u
	return u
}

// {sdkname}.install.toml file
func GetSDKInstallationConfFileBySDKName(sdkName string) string {
	host := os.Getenv(VMRInstallationHostEnv)
	if host == "" {
		host = DefaultInstallationHost
	}
	u, _ := url.JoinPath(host, fmt.Sprintf(SDKInstallationUrlPattern, sdkName))
	u = GetReverseProxyUri() + u
	return u
}
