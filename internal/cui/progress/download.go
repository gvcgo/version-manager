package progress

import (
	"fmt"
	"io"
	"os"
	"strings"

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

	return &Downloader{
		url:    uRL,
		client: request.New(),
		bar:    NewProgress(title),
	}
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
		go d.Start()
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

func (d *Downloader) Start() {
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
