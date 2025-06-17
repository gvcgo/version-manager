package progress

import (
	"context"
	"fmt"
	"io"
	"os"
	"strings"
	"sync"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/gvcgo/version-manager/internal/request"
	"github.com/gvcgo/version-manager/internal/utils"
)

type StartDownloadMsg struct{}

var startDownloadCmd = func() tea.Cmd {
	return func() tea.Msg {
		return StartDownloadMsg{}
	}
}

/*
single-threaded download.
*/
type Downloader struct {
	url        string
	client     *request.ReqClient
	bar        *Progress
	outputFile string
	nthreads   int64
	wg         *sync.WaitGroup
	context    context.Context
	cancel     context.CancelFunc
	tasks      []*PartialTask
}

func NewDownloader(uRL string) *Downloader {
	if uRL == "" {
		return nil
	}

	idx := strings.LastIndex(uRL, "/")
	if idx == -1 {
		return nil
	}
	title := fmt.Sprintf("Downloading %s", uRL[idx+1:])

	ctx := context.Background()
	ctx, cancel := context.WithCancel(ctx)

	r := request.New()
	r.SetContext(ctx)

	d := &Downloader{
		url:      uRL,
		client:   r,
		bar:      NewProgress(title),
		nthreads: 1,
		wg:       &sync.WaitGroup{},
		context:  ctx,
		cancel:   cancel,
	}

	d.AddOptions(WithCompleteHook(d.merge))
	d.AddOptions(WithCancelHook(func() error {
		d.cancel()
		time.Sleep(time.Millisecond * 500)
		if d.nthreads < 2 {
			return nil
		}
		return os.RemoveAll(d.getTempDir())
	}))
	return d
}

func (d *Downloader) SetOutputFilePath(fPath string) {
	d.outputFile = fPath
}

func (d *Downloader) AddOptions(opts ...any) {
	d.bar.AddOptions(opts...)
}

func (d *Downloader) GetTotalSize() int64 {
	resp, err := d.client.DoHead(d.url)
	if resp == nil || err != nil {
		return 0
	}
	contentLength := resp.Response.ContentLength
	resp.Body.Close()
	return contentLength
}

func (d *Downloader) Init() tea.Cmd {
	return startDownloadCmd()
}

func (d *Downloader) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case StartDownloadMsg:
		if d.nthreads < 2 {
			go d.StartSingle()
		} else {
			go d.StartMulti()
		}
		return d, nil
	default:
		return d.bar.Update(msg)
	}
}

func (d *Downloader) View() string {
	return d.bar.View()
}

func (d *Downloader) Help() string {
	return d.bar.Help()
}

func (d *Downloader) Write(partial []byte) (int, error) {
	return d.bar.Write(partial)
}

func (d *Downloader) StartSingle() {
	if utils.PathIsExist(d.outputFile) {
		err := os.RemoveAll(d.outputFile)
		if err != nil {
			d.bar.err = err
			return
		}
	}

	file, err := os.Create(d.outputFile)
	if err != nil {
		d.bar.err = err
		return
	}
	defer file.Close()
	w := io.MultiWriter(file, d)
	_, err = d.client.DoDownloadToWriter(w, d.url)
	d.bar.err = err
	return
}
