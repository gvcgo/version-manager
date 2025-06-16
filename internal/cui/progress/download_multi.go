package progress

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sync"

	"github.com/gvcgo/version-manager/internal/request"
	"github.com/gvcgo/version-manager/internal/utils"
)

/*
Multi-threaded downloading.
*/
func (d *Downloader) SetThreads(count int64) {
	d.nthreads = count
}

func (d *Downloader) getTempDir() string {
	parentDir := filepath.Dir(d.outputFile)
	tempDir := filepath.Join(parentDir, "temp-vmr")
	if !utils.PathIsExist(tempDir) {
		if err := os.MkdirAll(tempDir, os.ModePerm); err != nil {
			return ""
		}
	}
	return tempDir
}

func (d *Downloader) StartMulti() {
	if d.nthreads < 2 {
		return
	}
	total := d.bar.GetTotal()
	if total <= 0 {
		return
	}

	interval := total / d.nthreads
	if interval == 0 {
		interval = 1
	}

	var start int64 = 0
	num := 1
	var tasks []*PartialTask
	for start < total {
		end := start + interval
		end = min(end, total)
		fileName := filepath.Base(d.outputFile)
		fPath := filepath.Join(d.getTempDir(), fileName) + fmt.Sprintf(".part%d", num)
		pt := &PartialTask{
			url:             d.url,
			partialFilePath: fPath,
			byteFrom:        start,
			byteTo:          end,
			client:          request.New(),
			bar:             d.bar,
			wg:              d.wg,
			cancel:          d.cancel,
		}
		pt.SetContext(d.context)
		tasks = append(tasks, pt)
		num++
		start += interval
	}

	for _, task := range tasks {
		go task.Do()
	}
	d.wg.Wait()
	d.merge(tasks)
}

func (d *Downloader) merge(tasks []*PartialTask) {
	// TODO: merge
}

type PartialTask struct {
	url             string
	partialFilePath string
	byteFrom        int64
	byteTo          int64
	client          *request.ReqClient
	bar             *Progress
	wg              *sync.WaitGroup
	cancel          context.CancelFunc
}

func (p *PartialTask) SetContext(ctx context.Context) {
	p.client.SetContext(ctx)
}

func (p *PartialTask) Do() {
	p.wg.Add(1)
	defer p.wg.Done()
	if utils.PathIsExist(p.partialFilePath) {
		err := os.RemoveAll(p.partialFilePath)
		if err != nil {
			p.bar.err = err
			if p.cancel != nil {
				p.cancel()
			}
			return
		}
	}

	file, err := os.Create(p.partialFilePath)
	if err != nil {
		p.bar.err = err
		if p.cancel != nil {
			p.cancel()
		}
		return
	}
	defer file.Close()
	w := io.MultiWriter(file, p.bar)
	_, err = p.client.DoDownloadToWriter(w, p.url)
	p.bar.err = err
	if err != nil && p.cancel != nil {
		p.cancel()
	}
}
