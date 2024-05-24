package cnf

import (
	"os"
	"path/filepath"

	toml "github.com/pelletier/go-toml/v2"
)

const (
	DefaultReverseProxy   string = "https://gvc.1710717.xyz/proxy/"
	DefaultHostUrl        string = "https://raw.githubusercontent.com/gvcgo/vsources/main"
	SDKNameListFileUrl    string = `/sdk-list.version.json`
	VersionFileUrlPattern string = `/%s.version.json`
	VMRWorkDirName        string = ".vmr"
)

/*
Envs
*/
const (
	VMRSdkInstallationDirEnv string = "VMR_SDK_INSTALLATION_DIR"
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
	p := filepath.Join(GetVMRWorkDir(), "cache")
	os.MkdirAll(p, os.ModePerm)
	return p
}

func GetVersionsDir() string {
	sdkInstallationDir := os.Getenv(VMRSdkInstallationDirEnv)
	if sdkInstallationDir != "" {
		vp := filepath.Join(sdkInstallationDir, "versions")
		os.MkdirAll(vp, os.ModePerm)
		return vp
	} else {
		vp := filepath.Join(GetVMRWorkDir(), "versions")
		os.MkdirAll(vp, os.ModePerm)
		return vp
	}
}

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
