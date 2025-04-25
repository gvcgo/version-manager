package extract

import (
	"context"
	"errors"
	"fmt"
	"os"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/mholt/archives"
)

/*
Extract archived or compressed files.
*/
type Extractor struct {
	Source      string
	Destination string
}

func New(src, dest string) *Extractor {
	return &Extractor{
		Source:      src,
		Destination: dest,
	}
}

func (e *Extractor) Extract() error {
	if ok, _ := gutils.PathIsExist(e.Source); !ok {
		return errors.New("source file does not exist")
	}
	file, err := os.Open(e.Source)
	if err != nil {
		return err
	}
	defer file.Close()

	ctx := context.TODO()

	format, _, err := archives.Identify(ctx, e.Source, nil)
	if err != nil {
		return err
	}

	if decomp, ok := format.(archives.Decompressor); ok {
		rc, err := decomp.OpenReader(file)
		if err != nil {
			return err
		}
		fmt.Println(rc)
	}

	return nil
}

func (e *Extractor) Unarchive(name string) error {

	return nil
}
