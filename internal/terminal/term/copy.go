//go:build darwin || freebsd || dragonfly || linux

package term

import (
	"io"
	"os"
	"syscall"
)

func FD_SET(p *syscall.FdSet, fd int) {
	p.Bits[fd/32] |= 1 << uint(fd) % 32
}

func FD_ISSET(p *syscall.FdSet, fd int) bool {
	return (p.Bits[fd/32] & (1 << uint(fd) % 32)) != 0
}

func Select(nfd int, r *syscall.FdSet, w *syscall.FdSet, e *syscall.FdSet, timeout *syscall.Timeval) error {
	return syscall.Select(nfd, r, w, e, timeout)
}

func Copy(dst io.Writer, src *os.File) func() {
	r, w, _ := os.Pipe()

	go func() {
		copy(dst, src, r)
	}()

	return func() {
		w.Write([]byte("x"))
		r.Close()
		w.Close()
	}
}

func copy(dst io.Writer, src *os.File, finish *os.File) (written int64, err error) {
	fd := int(src.Fd())
	ffd := int(finish.Fd())
	maxfd := ffd + 1
	if fd > ffd {
		maxfd = fd + 1
	}
	rfds := &syscall.FdSet{}
	buf := make([]byte, 32*1024)

	for {
		FD_SET(rfds, fd)
		FD_SET(rfds, ffd)

		es := Select(maxfd, rfds, nil, nil, nil)
		if es != nil {
			if es == syscall.EINTR {
				continue
			}

			err = es
			break
		}

		if FD_ISSET(rfds, fd) {
			nr, er := src.Read(buf)
			if nr > 0 {
				nw, ew := dst.Write(buf[0:nr])
				if nw > 0 {
					written += int64(nw)
				}
				if ew != nil {
					err = ew
					break
				}
				if nr != nw {
					err = io.ErrShortWrite
					break
				}
			}
			if er == io.EOF {
				break
			}
			if er != nil {
				err = er
				break
			}
		}

		if FD_ISSET(rfds, ffd) {
			break
		}
	}

	return written, err
}
