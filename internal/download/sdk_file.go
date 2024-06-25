package download

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
)

/*
Download SDK files to cache dir.
<sdk_installation_dir>/cache/<SDKName>/<VersionName>
*/

type Downloader struct {
	Fetcher     *request.Fetcher
	SDKName     string
	VersionName string
	Version     Item
}

func NewDownloader() (d *Downloader) {
	return &Downloader{
		Fetcher: request.NewFetcher(),
	}
}

func (d *Downloader) getLocalFilePath() string {
	cacheDir := cnf.GetCacheDir()
	filename := filepath.Base(d.Version.Url)
	// gradle: https://raw.githubusercontent.com/gvcgo/vsources/main/gradle.version.json
	if d.SDKName == "gradle" && strings.Contains(filename, "?") {
		filename = fmt.Sprintf("gradle-%s-all.zip", d.VersionName)
	}
	dd := filepath.Join(cacheDir, d.SDKName, d.VersionName)
	os.MkdirAll(dd, os.ModePerm)
	return filepath.Join(dd, filename)
}

func (d *Downloader) Download(OriginSDKName, versionName string, version Item, force ...bool) (fPath string) {
	if version.Url == "" {
		return
	}
	d.SDKName = OriginSDKName
	d.VersionName = versionName
	d.Version = version

	fPath = d.getLocalFilePath()
	if ok, _ := gutils.PathIsExist(fPath); ok {
		return
	}

	d.Fetcher = cnf.GetFetcher(d.Version.Url)
	// d.Fetcher.SetUrl(d.Version.Url)
	// d.Fetcher.Proxy = "http://localhost:2023"
	d.Fetcher.Timeout = 30 * time.Minute
	d.Fetcher.SetCheckSum(version.Sum, version.SumType)
	d.Fetcher.SetFileContentLength(version.Size)

	if size := d.Fetcher.GetAndSaveFile(fPath, force...); size <= 100 {
		os.RemoveAll(fPath)
		return ""
	}
	return
}
