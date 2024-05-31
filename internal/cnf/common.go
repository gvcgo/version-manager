package cnf

import (
	"fmt"
	"net/url"
	"os"
	"strings"

	"github.com/gvcgo/goutils/pkgs/request"
)

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

// Prepares request.Fetcher for URL.
func GetFetcher(dUrl string) (fetcher *request.Fetcher) {
	reverseProxy := GetReverseProxyUri()
	localProxy := os.Getenv(VMRLocalProxyEnv)
	if localProxy == "" && !strings.Contains(dUrl, "gitee.com") {
		dUrl = reverseProxy + dUrl
	}
	fetcher = request.NewFetcher()
	fetcher.SetUrl(dUrl)
	if !strings.Contains(dUrl, "gitee.com") {
		fetcher.Proxy = localProxy
	}
	return
}
