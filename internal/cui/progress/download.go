package progress

import "github.com/imroc/req/v3"

/*
single-threaded download.
*/
type Downloader struct {
	url    string
	client *req.Client
	bar    *Progress
}
