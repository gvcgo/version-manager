package plugin

import (
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/goutils/pkgs/request"
	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/gvcgo/version-manager/internal/luapi/lua_global/gh"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
Download plugins from https://github.com/gvcgo/vmr_plugins.
*/
const (
	PluginsDownloadUrl string = "https://github.com/gvcgo/vmr_plugins/archive/refs/heads/main.zip"
	PluginRepo         string = "gvcgo/vmr_plugins"
	PluginInfoFileName string = "plugins.json"
)

func downloadPlugins() error {
	f := request.NewFetcher()
	cfg := cnf.NewVMRConf()
	dUrl := PluginsDownloadUrl
	if cfg.ProxyUri != "" {
		f.Proxy = cfg.ProxyUri
	} else if cfg.ReverseProxy != "" {
		dUrl = strings.TrimSuffix(cfg.ReverseProxy, "/") + "/" + dUrl
	}

	f.SetUrl(dUrl)
	destDir := cnf.GetTempDir()

	fName := filepath.Base(PluginsDownloadUrl)
	fPath := filepath.Join(destDir, fName)
	if size := f.GetFile(fPath, true); size < 100 {
		return errors.New("download failed")
	}

	return utils.Extract(fPath, destDir)
}

func copyPlugins() {
	pluginsDir := cnf.GetPluginDir()

	ff := utils.NewFinder("go.lua", "LICENSE")
	ff.SetFlagDirExcepted(true)
	ff.Find(cnf.GetTempDir())
	srcDir := ff.GetDirName()

	fileList, _ := os.ReadDir(srcDir)
	for _, f := range fileList {
		if f.IsDir() || !strings.HasSuffix(f.Name(), ".lua") {
			continue
		}
		srcPath := filepath.Join(srcDir, f.Name())
		destPath := filepath.Join(pluginsDir, f.Name())
		if ok, _ := gutils.PathIsExist(destPath); ok {
			os.RemoveAll(destPath)
		}
		gutils.CopyAFile(srcPath, destPath)
	}
}

func GetPluginFileList() []gh.RepoFile {
	cfg := cnf.NewVMRConf()
	cfg.Load()

	if cfg.GithubToken == "" {
		cfg.GithubToken = gh.GetDefaultReadOnly()
	}

	github := gh.NewGh(PluginRepo, cfg.GithubToken, cfg.ProxyUri, cfg.ReverseProxy)
	return github.GetFileList()
}

func updateInfo() {
	fileList := GetPluginFileList()
	result, _ := json.Marshal(fileList)
	infoFilePath := filepath.Join(cnf.GetPluginDir(), PluginInfoFileName)
	os.WriteFile(infoFilePath, result, os.ModePerm)
}

func UpdatePlugins() error {
	if err := downloadPlugins(); err != nil {
		return err
	}
	copyPlugins()
	updateInfo()
	os.RemoveAll(cnf.GetTempDir())
	return nil
}
