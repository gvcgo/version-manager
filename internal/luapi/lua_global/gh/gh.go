package gh

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"github.com/gvcgo/goutils/pkgs/crypt"
	"github.com/gvcgo/version-manager/internal/request"
)

func GetDefaultReadOnly() string {
	r := crypt.DecodeBase64("WjJod1gxY3lVV1paTVZrNVYyVnZNVXQxVDFKUVNGWkhTalZTTWtaemJuVnNNakZYVUVsQk1R")
	r = crypt.DecodeBase64(r)
	return r
}

// ReleaseItem
type Asset struct {
	Name      string    `json:"name"`
	Url       string    `json:"browser_download_url"`
	Size      int64     `json:"size"`
	CreatedAt time.Time `json:"created_at"`
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
	client       *request.ReqClient
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
	if g.client == nil {
		g.client = request.New()
		g.client.SetCommonHeader("Accept", AcceptHeader)
		g.client.SetCommonHeader("Authorization", fmt.Sprintf(AuthorizationHeader, g.Token))
	}

	// https://api.github.com/repos/{owner}/{repo}/releases?per_page=100&page=1
	dUrl := fmt.Sprintf(GithubReleaseAPI, GithubAPI, g.RepoName, page)

	resp, err := g.client.DoGet(dUrl)
	if err != nil {
		//TODO: handle error
		return
	}

	return resp.Bytes()
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

type RepoFile struct {
	Name string `json:"name"`
	Size int64  `json:"size"`
	Url  string `json:"download_url"`
	Sha  string `json:"sha"`
}

func (g *Gh) getFileList() (r []byte) {
	if g.client == nil {
		g.client = request.New()
		g.client.SetCommonHeader("Accept", AcceptHeader)
		g.client.SetCommonHeader("Authorization", fmt.Sprintf(AuthorizationHeader, g.Token))
	}
	//   https://api.github.com/repos/{gvcgo/vmr_plugins}/contents/
	dUrl := fmt.Sprintf("https://api.github.com/repos/%s/contents/", strings.Trim(g.RepoName, "/"))

	resp, err := g.client.DoGet(dUrl)
	if err != nil {
		//TODO: handle error
		return
	}
	return resp.Bytes()
}

/*
Github files.
*/
func (g *Gh) GetFileList() (files []RepoFile) {
	r := g.getFileList()
	json.Unmarshal(r, &files)
	return
}
