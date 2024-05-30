package cnf

import (
	"os"
	"path/filepath"

	toml "github.com/pelletier/go-toml/v2"
)

var DefaultConfig *VMRConf

func init() {
	DefaultConfig = NewVMRConf()
}

const (
	DefaultReverseProxy       string = "https://gvc.1710717.xyz/proxy/"
	DefaultHostUrl            string = "https://raw.githubusercontent.com/gvcgo/vsources/main"
	SDKNameListFileUrl        string = `/sdk-list.version.json`
	VersionFileUrlPattern     string = `/%s.version.json`
	SDKInstallationUrlPattern string = `install/%s.toml`
	VMRWorkDirName            string = ".vmr"
)

/*
Envs
*/
const (
	VMRSdkInstallationDirEnv string = "VMR_SDK_INSTALLATION_DIR"
	VMRHostUrlEnv            string = "VMR_HOST"
	VMRReverseProxyEnv       string = "VMR_REVERSE_PROXY"
	VMRLocalProxyEnv         string = "VMR_LOCAL_PROXY"
)

/*
vmr work dir:

where vmr is installed.
*/
func GetVMRWorkDir() string {
	homeDir, _ := os.UserHomeDir()
	p := filepath.Join(homeDir, VMRWorkDirName)
	os.MkdirAll(p, os.ModePerm)
	return p
}

/*
vmr conf file path.
*/
func GetVMRConfFilePath() string {
	return filepath.Join(GetVMRWorkDir(), "conf.toml")
}

/*
versions dir:

where the versions are installed.
*/
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
cache file dir:

where the downloaded files are stored.
*/
func GetCacheDir() string {
	sdkInstallationDir := filepath.Dir(GetVersionsDir())
	p := filepath.Join(sdkInstallationDir, "cache")
	os.MkdirAll(p, os.ModePerm)
	return p
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

/*
==============================
vmr config file.
==============================
*/
type VMRConf struct {
	ProxyUri          string `json,toml:"proxy_uri"`
	ReverseProxy      string `json,toml:"reverse_proxy"`
	SDKIntallationDir string `json,toml:"sdk_installation_dir"`
	VersionHostUrl    string `json,toml:"version_host_url"`
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
	if v.ProxyUri != "" {
		os.Setenv(VMRLocalProxyEnv, v.ProxyUri)
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

func (v *VMRConf) SetVersionHostUrl(hUrl string) {
	if hUrl == "" {
		return
	}
	v.Load()
	v.VersionHostUrl = hUrl
	v.Save()
}
