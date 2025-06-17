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
	os.RemoveAll(d.getTempDir()) // try to remove old temp files

	interval := total / d.nthreads
	if interval == 0 {
		interval = 1
	}

	var start int64 = 0
	if d.tasks == nil {
		d.tasks = []*PartialTask{}
	}

	for i := range d.nthreads {
		// concurrency request, i for thread id
		end := start + interval
		fileName := filepath.Base(d.outputFile)
		fPath := filepath.Join(d.getTempDir(), fileName) + fmt.Sprintf(".part%d", i+1)
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
		d.tasks = append(d.tasks, pt)

		start += interval + 1
	}

	for _, task := range d.tasks {
		go task.Do()
	}
	d.wg.Wait()
}

func (d *Downloader) merge() error {
	if d.nthreads < 2 {
		return nil
	}
	dest_file, err := os.OpenFile(d.outputFile, os.O_CREATE|os.O_WRONLY, os.ModePerm)
	if err != nil {
		return err
	}
	defer utils.Closeq(dest_file)

	for _, t := range d.tasks {
		part_file, err := os.Open(t.GetPartialFilePath())
		if err != nil {
			return err
		}
		io.Copy(dest_file, part_file)
		utils.Closeq(part_file)
	}

	os.RemoveAll(d.getTempDir())
	return nil
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
	p.client.SetCommonHeader("Range", fmt.Sprintf("bytes=%d-%d", p.byteFrom, p.byteTo))
	_, err = p.client.DoDownloadToWriter(w, p.url)
	p.bar.err = err
	if err != nil && p.cancel != nil {
		p.cancel()
	}
}

func (p *PartialTask) GetPartialFilePath() string {
	return p.partialFilePath
}
