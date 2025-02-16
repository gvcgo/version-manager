package cnf

import (
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/pelletier/go-toml/v2"
)

// sdk-list.version.json
func GetSDKListFileUrl() string {
	host := os.Getenv(VMRHostUrlEnv)
	if host == "" {
		host = DefaultHostUrl
	}
	u, _ := url.JoinPath(host, SDKNameListFileUrl)
	// u = GetReverseProxyUri() + u
	return u
}

// {sdkname}.version.json file
func GetVersionFileUrlBySDKName(sdkName string) string {
	host := os.Getenv(VMRHostUrlEnv)
	if host == "" {
		host = DefaultHostUrl
	}
	u, _ := url.JoinPath(host, fmt.Sprintf(VersionFileUrlPattern, sdkName))
	// u = GetReverseProxyUri() + u
	return u
}

// install/{sdkname}.toml file
func GetSDKInstallationConfFileUrlBySDKName(sdkName string) string {
	host := os.Getenv(VMRHostUrlEnv)
	if host == "" {
		host = DefaultHostUrl
	}
	u, _ := url.JoinPath(host, fmt.Sprintf(SDKInstallationUrlPattern, sdkName))
	// u = GetReverseProxyUri() + u
	return u
}

// reverse proxy
func GetReverseProxyUri(dUrl, localProxy string) string {
	if localProxy != "" {
		// use localProxy prior to reverseProxy
		return ""
	}
	if strings.Contains(dUrl, "gitee.com") {
		// gittee does not need reverseProxy
		return ""
	}
	rp := os.Getenv(VMRReverseProxyEnv)
	if rp == "" && strings.Contains(dUrl, "github") {
		rp = DefaultReverseProxy
	}

	if !strings.HasSuffix(rp, "/") {
		rp = rp + "/"
	}
	return rp
}

// Get download thread num
func GetDownloadThreadNum() int {
	threadNum := os.Getenv(VMRDonwloadThreadEnv)
	num := gconv.Int(threadNum)
	if num < 1 {
		num = 1
	}
	return num
}

func LoadCustomedMirror() map[string]string {
	result := make(map[string]string)
	fPath := filepath.Join(GetVMRWorkDir(), "customed_mirrors.toml")
	if ok, _ := gutils.PathIsExist(fPath); !ok {
		ff := request.NewFetcher()
		ff.SetUrl(DefaultReverseProxy + strings.TrimSuffix(DefaultHostUrl, "/") + "/mirrors/customed_mirrors.toml")
		s, _ := ff.GetString()
		os.WriteFile(fPath, []byte(s), os.ModePerm)
	}
	content, _ := os.ReadFile(fPath)
	toml.Unmarshal(content, &result)
	return result
}

func UseCustomedMirrorUrl(dUrl string) string {
	if !gconv.Bool(os.Getenv(VMRUseCustomedMirrorEnv)) {
		return dUrl
	}
	mirrors := LoadCustomedMirror()
	for kk, vv := range mirrors {
		if strings.Contains(dUrl, kk) {
			if strings.HasPrefix(dUrl, "https://gradle.org/releases") && strings.Contains(vv, `%s`) {
				uu, err := url.Parse(dUrl)
				if err != nil {
					return dUrl
				}
				version := uu.Query().Get("version")
				if version == "" {
					return dUrl
				}
				return fmt.Sprintf(vv, version)
			} else {
				dUrl = strings.ReplaceAll(dUrl, kk, vv)
			}
		}
	}
	return dUrl
}

// Prepares request.Fetcher for URL.
func GetFetcher(dUrl string) (fetcher *request.Fetcher) {
	// use customed mirror
	oldDUrl := dUrl
	dUrl = UseCustomedMirrorUrl(dUrl)

	localProxy := os.Getenv(VMRLocalProxyEnv)
	reverseProxy := strings.Trim(GetReverseProxyUri(dUrl, localProxy), "/")
	if reverseProxy != "" && oldDUrl == dUrl {
		dUrl = reverseProxy + "/" + dUrl
	}
	fetcher = request.NewFetcher()

	// multi-threads only for large files.
	if !strings.HasSuffix(dUrl, ".json") && !strings.HasSuffix(dUrl, ".toml") {
		fetcher.SetThreadNum(GetDownloadThreadNum())
	}

	fetcher.SetUrl(strings.Trim(dUrl, "/"))
	if !strings.Contains(dUrl, "gitee.com") && oldDUrl == dUrl {
		// do not use proxy for gitee.
		fetcher.Proxy = localProxy
	}
	return
}

func GetGithubToken() string {
	cnf := NewVMRConf()
	cnf.Load()
	return cnf.GithubToken
}

func GetCacheRetentionTime() int64 {
	cnf := NewVMRConf()
	cnf.Load()
	if cnf.CacheRetentionTime == 0 {
		return 86400
	}
	return cnf.CacheRetentionTime
}
