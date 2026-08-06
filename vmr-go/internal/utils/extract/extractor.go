package extract

import (
	"context"
	"errors"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/mholt/archives"
)

/*
Extract archived or compressed files.
*/
type Extractor struct {
	Source                   string
	Destination              string
	ArchivePath              string
	isCompressedSingleExe    bool
	compressedSingleFileName string
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

func (e *Extractor) baseName() string {
	baseName := filepath.Base(e.Source)
	if strings.Contains(baseName, ".") {
		index := strings.LastIndex(baseName, ".")
		baseName = baseName[:index]
	}
	return baseName
}

func (e *Extractor) SetCompressedSingleExe() {
	e.isCompressedSingleExe = true
}

func (e *Extractor) SetCompressedSingleFileName(name string) {
	e.compressedSingleFileName = name
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

	tmpDir := e.tmpDir()
	e.ArchivePath = filepath.Join(tmpDir, e.baseName())

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
		f, err := info.Open()
		if err != nil {
			return err
		}
		defer f.Close()

		if info.IsDir() {
			return nil
		}

		name := info.NameInArchive
		if strings.Contains(name, string(filepath.Separator)) {
			dir := filepath.Dir(name)
			if dir != "" {
				_ = os.MkdirAll(filepath.Join(e.Destination, dir), os.ModePerm)
			}
		}

		newFilePath := filepath.Join(e.Destination, name)
		out, err := os.Create(newFilePath)
		if err != nil {
			return err
		}

		_, err = io.Copy(out, f)
		if err != nil {
			return err
		}

		return os.Chmod(newFilePath, info.Mode())
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
		defer func() {
			_ = os.RemoveAll(e.tmpDir())
		}()
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
		// Handle compressed single file.
		if strings.Contains(err.Error(), "no formats") {
			name := e.baseName()
			src := filepath.Join(e.tmpDir(), name)
			dst := filepath.Join(e.Destination, name)
			if e.compressedSingleFileName != "" {
				dst = filepath.Join(e.Destination, e.compressedSingleFileName)
			}

			err := gutils.CopyAFile(src, dst)
			if err != nil {
				return err
			}

			if e.isCompressedSingleExe {
				info, err := os.Stat(dst)
				if err != nil {
					return err
				}
				newMode := info.Mode() | 0111
				return os.Chmod(dst, newMode)
			}
			return nil
		}
		return err
	}

	if extractor, ok := format.(archives.Extractor); ok {
		err := e.Extract(extractor, file)
		if err != nil {
			return err
		}
	}

	return nil
}
