package luapi

import (
	"os"
	"strings"
	"time"

	"github.com/PuerkitoBio/goquery"
	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gvcgo/goutils/pkgs/request"
)

const (
	ProxyEnvName string = "VCOLLECTOR_PROXY"
)

func GetResp(dUrl string, timeout ...int) string {
	t := 180
	if len(timeout) > 0 && timeout[0] > 0 {
		t = timeout[0]
	}

	fetcher := request.NewFetcher()
	fetcher.SetUrl(dUrl)
	fetcher.Timeout = time.Duration(t) * time.Second

	proxy := os.Getenv(ProxyEnvName)
	// if gconv.Bool(proxy) && !strings.Contains(dUrl, "maven") && !strings.Contains(dUrl, "android") {
	if gconv.Bool(proxy) {
		fetcher.Proxy = proxy
	}
	// fetcher.Proxy = "http://127.0.0.1:2023"

	resp, code := fetcher.GetString()
	if code != 200 || resp == "" {
		return ""
	}
	return resp
}

func GetDocument(dUrl string, timeout ...int) *goquery.Document {
	if resp := GetResp(dUrl, timeout...); resp != "" {
		doc, _ := goquery.NewDocumentFromReader(strings.NewReader(resp))
		return doc
	}
	return nil
}

type Crawler struct {
	Url     string
	Timeout int // in seconds
}

func NewCrawler(url string, timeout int) *Crawler {
	return &Crawler{
		Url:     url,
		Timeout: timeout,
	}
}
