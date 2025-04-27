package extract

import (
	"context"
	"errors"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/mholt/archives"
)

/*
Extract archived or compressed files.
*/
type Extractor struct {
	Source      string
	Destination string
	ArchivePath string
}

func New(src, dest string) *Extractor {
	return &Extractor{
		Source:      src,
		Destination: dest,
	}
}

func (e *Extractor) tmpDir() string {
	tmpDir := filepath.Join(e.Destination, "temp")
	_ = os.MkdirAll(tmpDir, os.ModePerm)
	return tmpDir
}

func (e *Extractor) Decompress(decomp archives.Decompressor, file *os.File) error {
	if file == nil {
		return errors.New("file is nil")
	}
	defer file.Close()

	if decomp == nil {
		return errors.New("decompressor is nil")
	}

	rc, err := decomp.OpenReader(file)
	if err != nil {
		return err
	}

	baseName := filepath.Base(e.Source)
	if strings.Contains(baseName, ".") {
		index := strings.LastIndex(baseName, ".")
		baseName = baseName[:index]
	}

	tmpDir := e.tmpDir()
	e.ArchivePath = filepath.Join(tmpDir, baseName)

	f, err := os.Create(e.ArchivePath)
	if err != nil {
		return err
	}
	defer f.Close()

	_, err = io.Copy(f, rc)
	return err
}

func (e *Extractor) Extract(extractor archives.Extractor, file *os.File) error {
	if file == nil {
		return errors.New("file is nil")
	}
	if extractor == nil {
		return errors.New("extractor is nil")
	}

	extractor.Extract(context.TODO(), file, func(ctx context.Context, info archives.FileInfo) error {
		// TODO: write file.
		return nil
	})
	return nil
}

func (e *Extractor) Unarchive() error {
	if e.Source == "" {
		return errors.New("source is empty")
	}

	file, err := os.Open(e.Source)
	if err != nil {
		return err
	}

	format, _, err := archives.Identify(context.TODO(), e.Source, file)
	if err != nil {
		return err
	}

	if decomp, ok := format.(archives.Decompressor); ok {
		err := e.Decompress(decomp, file)
		if err != nil {
			return err
		}
	} else {
		e.ArchivePath = e.Source
	}

	file, err = os.Open(e.ArchivePath)
	if err != nil {
		return err
	}
	defer file.Close()

	format, _, err = archives.Identify(context.TODO(), e.ArchivePath, file)
	if err != nil {
		return err
	}

	if extractor, ok := format.(archives.Extractor); ok {
		err := e.Extract(extractor, file)
		if err != nil {
			return err
		}
	}

	_ = os.RemoveAll(e.tmpDir())
	return nil
}
