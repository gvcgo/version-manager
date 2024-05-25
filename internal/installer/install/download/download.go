package download

import (
	"path/filepath"
	"time"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
Download SDK files to cache dir.
<sdk_installation_dir>/cache/<SDKName>/<VersionName>
*/

type Downloader struct {
	Fetcher     *request.Fetcher
	SDKName     string
	VersionName string
	Version     utils.Item
}

func NewDownloader() (d *Downloader) {
	return &Downloader{
		Fetcher: request.NewFetcher(),
	}
}

func (d *Downloader) getLocalFilePath() string {
	cacheDir := cnf.GetCacheDir()
	filename := filepath.Base(d.Version.Url)
	return filepath.Join(cacheDir, d.SDKName, d.VersionName, filename)
}

func (d *Downloader) Download(OriginSDKName, versionName string, version utils.Item) {
	if version.Url == "" {
		return
	}
	d.SDKName = OriginSDKName
	d.VersionName = versionName
	d.Version = version

	fName := d.getLocalFilePath()
	if ok, _ := gutils.PathIsExist(fName); ok {
		return
	}
	d.Fetcher.SetUrl(cnf.GetReverseProxyUri() + d.Version.Url)
	d.Fetcher.Timeout = 30 * time.Minute
	d.Fetcher.GetAndSaveFile(d.getLocalFilePath(), false)
}
