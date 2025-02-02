package gh

import (
	"encoding/json"
	"fmt"
	"io"
	"net/url"
	"time"

	"github.com/gvcgo/goutils/pkgs/request"
)

// ReleaseItem
type Asset struct {
	Name string `json:"name"`
	Url  string `json:"browser_download_url"`
	Size int64  `json:"size"`
}

type ReleaseItem struct {
	Assets     []Asset `json:"assets"`
	TagName    string  `json:"tag_name"`
	PreRelease any     `json:"prerelease"`
}

type ReleaseList []ReleaseItem

/*
Get github releases
*/

const (
	GithubAPI           string = "https://api.github.com"
	GithubReleaseAPI    string = "%s/repos/%s/releases?per_page=100&page=%d"
	AcceptHeader        string = "application/vnd.github.v3+json"
	AuthorizationHeader string = "token %s"
)

type Gh struct {
	RepoName     string
	Token        string
	Proxy        string
	ReverseProxy string
	fetcher      *request.Fetcher
}

func NewGh(repo, token, proxy, reverseProxy string) (g *Gh) {
	g = &Gh{
		RepoName:     repo,
		Token:        token,
		Proxy:        proxy,
		ReverseProxy: reverseProxy,
	}
	return
}

func (g *Gh) getRelease(page int) (r []byte) {
	if g.fetcher == nil {
		g.fetcher = request.NewFetcher()
		g.fetcher.Headers = map[string]string{
			"Accept":        AcceptHeader,
			"Authorization": fmt.Sprintf(AuthorizationHeader, g.Token),
		}
		// TODO: debug only
		g.Proxy = "http://127.0.0.1:2023"
	}

	// https://api.github.com/repos/{owner}/{repo}/releases?per_page=100&page=1
	dUrl := fmt.Sprintf(GithubReleaseAPI, GithubAPI, g.RepoName, page)

	if g.Proxy != "" {
		g.fetcher.Proxy = g.Proxy
	} else if g.ReverseProxy != "" {
		dUrl, _ = url.JoinPath(g.ReverseProxy, dUrl)
	}

	g.fetcher.SetUrl(dUrl)
	g.fetcher.Timeout = 180 * time.Second
	if resp := g.fetcher.Get(); resp != nil {
		defer resp.RawResponse.Body.Close()
		r, _ = io.ReadAll(resp.RawResponse.Body)
	}
	return
}

func (g *Gh) GetReleases() (rl ReleaseList) {
	page := 1
	for {
		itemList := ReleaseList{}
		r := g.getRelease(page)
		json.Unmarshal(r, &itemList)
		if len(itemList) == 0 || page >= 10 {
			break
		}
		rl = append(rl, itemList...)
		page++
	}
	return
}
