//go:build windows

package term

import (
	"context"
	"io"
	"log"
	"os"
)

/*
https://github.com/marcomorain/go-conpty
https://github.com/ActiveState/termtest
*/

type Pty struct {
	Stdin  *os.File
	Stdout *os.File
}

func NewTerminal() Terminal {
	return &Pty{Stdin: os.Stdin, Stdout: os.Stdout}
}

func (p *Pty) Size() (rows, cols int, err error) {
	coord, err := WinConsoleScreenSize()
	return coord.Y, coord.X, err
}

func (p *Pty) Record(command string, envs ...string) error {
	height, width, _ := p.Size()
	if width == 0 {
		width = 180
	}
	if height == 0 {
		height = 100
	}

	cpty, err := Start(command, &COORD{X: width, Y: height}, envs)
	if err != nil {
		return err
	}
	defer cpty.Close()

	go func() {
		go io.Copy(p.Stdout, cpty)
		io.Copy(cpty, p.Stdin)
	}()

	exitCode, err := cpty.Wait(context.Background())
	if err != nil {
		log.Fatalf("Error: %v", err)
	}
	log.Printf("ExitCode: %d", exitCode)
	return nil
}
