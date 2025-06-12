package request

import (
	"context"
	"errors"
	"io"
	"strings"
	"time"

	"github.com/gvcgo/version-manager/internal/cnf"
	"github.com/imroc/req/v3"
)

/*
http request.
*/
type ReqClient struct {
	*req.Client
	cfg *cnf.VMRConf
}

func New() *ReqClient {
	cfg := cnf.NewVMRConf()
	cfg.Load()
	rc := &ReqClient{
		Client: req.C(),
		cfg:    cfg,
	}
	return rc.UseDefaultProxy().UseDefaultAgent()
}

func (rc *ReqClient) UseDefaultProxy() *ReqClient {
	if rc.cfg.ProxyUri != "" {
		rc.Client = rc.Client.SetProxyURL(rc.cfg.ProxyUri)
	}
	return rc
}

func (rc *ReqClient) UseDefaultAgent() *ReqClient {
	rc.Client = rc.Client.SetCommonHeader(
		"User-Agent",
		"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36",
	)
	return rc
}

func (rc *ReqClient) UseDefaultTimeout() *ReqClient {
	t := time.Minute * 30
	rc.Client = rc.Client.SetTimeout(t)
	return rc
}

func (rc *ReqClient) UseDefaultRetry() *ReqClient {
	rc.Client = rc.Client.SetCommonRetryCount(2)
	return rc
}

func (rc *ReqClient) tryToUseReverseProxy(rawUrl string) string {
	if rc.cfg.ReverseProxy == "" {
		return rawUrl
	}
	return strings.TrimSuffix(rc.cfg.ReverseProxy, "/") + "/" + rawUrl
}

func (rc *ReqClient) DoHead(url ...string) (*req.Response, error) {
	var resp *req.Response
	if len(url) > 0 {
		rawURL := rc.tryToUseReverseProxy(url[0])
		resp = rc.Client.Head(rawURL).Do(context.TODO())
	} else {
		resp = rc.Client.Head().Do(context.TODO())
	}
	if resp == nil {
		return nil, errors.New("nil response")
	}
	return resp, resp.Err
}

func (rc *ReqClient) DoDownloadToWriter(w io.Writer, url string) (*req.Response, error) {
	rawURL := rc.tryToUseReverseProxy(url)
	return rc.Client.R().SetOutput(w).Get(rawURL)
}
