//go:build windows

package winpty

import (
	"errors"
	"fmt"
	"strings"
	"syscall"
	"unicode/utf16"
	"unsafe"

	"golang.org/x/sys/windows"
)

var (
	modKernel32                        = windows.NewLazySystemDLL("kernel32.dll")
	fCreatePseudoConsole               = modKernel32.NewProc("CreatePseudoConsole")
	fResizePseudoConsole               = modKernel32.NewProc("ResizePseudoConsole")
	fClosePseudoConsole                = modKernel32.NewProc("ClosePseudoConsole")
	fInitializeProcThreadAttributeList = modKernel32.NewProc("InitializeProcThreadAttributeList")
	fUpdateProcThreadAttribute         = modKernel32.NewProc("UpdateProcThreadAttribute")
	fPeekNamedPipe                     = modKernel32.NewProc("PeekNamedPipe")
	ErrConPtyUnsupported               = errors.New("ConPty is not available on this version of Windows")
)

const (
	S_OK uintptr = 0
)

type COORD struct {
	X, Y int
}

func (c *COORD) Pack() uintptr {
	return uintptr((int32(c.Y) << 16) | int32(c.X))
}

func Win32CreatePseudoConsole(coord *COORD, hIn, hOut windows.Handle) (windows.Handle, error) {
	if fCreatePseudoConsole.Find() != nil {
		return 0, fmt.Errorf("createPseudoConsole not found")
	}
	var hPc windows.Handle
	ret, _, _ := fCreatePseudoConsole.Call(
		coord.Pack(),
		uintptr(hIn),
		uintptr(hOut),
		0,
		uintptr(unsafe.Pointer(&hPc)))
	if ret != S_OK {
		return 0, fmt.Errorf("createPseudoConsole() failed with status 0x%x", ret)
	}
	return hPc, nil
}

func Win32ClosePseudoConsole(hPc windows.Handle) {
	if fClosePseudoConsole.Find() != nil {
		return
	}
	// this kills the attached process. there is no return value.
	fClosePseudoConsole.Call(uintptr(hPc))
}

func Win32ResizePseudoConsole(hPc windows.Handle, coord *COORD) error {
	if fResizePseudoConsole.Find() != nil {
		return fmt.Errorf("ResizePseudoConsole not found")
	}
	ret, _, _ := fResizePseudoConsole.Call(uintptr(hPc), coord.Pack())
	if ret != S_OK {
		return fmt.Errorf("ResizePseudoConsole failed with status 0x%x", ret)
	}
	return nil
}

func WinConsoleScreenSize() (size *COORD, err error) {
	// Determine required size of Pseudo Console
	consoleSize := new(COORD)
	var csbi windows.ConsoleScreenBufferInfo

	console, err := windows.GetStdHandle(windows.STD_OUTPUT_HANDLE)
	if err != nil {
		return nil, err
	}

	err = windows.GetConsoleScreenBufferInfo(console, &csbi)
	if err != nil {
		return nil, err
	}

	consoleSize.X = int(csbi.Window.Right - csbi.Window.Left + 1)
	consoleSize.Y = int(csbi.Window.Bottom - csbi.Window.Top + 1)
	return consoleSize, nil
}

// This will only return the first error.
func WinCloseHandles(handles ...windows.Handle) error {
	var err error
	for _, h := range handles {
		if h != windows.InvalidHandle {
			if err == nil {
				err = windows.CloseHandle(h)
			} else {
				windows.CloseHandle(h)
			}
		}
	}
	return err
}

func WinIsConPtyAvailable() bool {
	return fCreatePseudoConsole.Find() == nil &&
		fResizePseudoConsole.Find() == nil &&
		fClosePseudoConsole.Find() == nil &&
		fInitializeProcThreadAttributeList.Find() == nil &&
		fUpdateProcThreadAttribute.Find() == nil
}

func EnableVirtualTerminalProcessing() error {
	console, err := windows.GetStdHandle(windows.STD_OUTPUT_HANDLE)
	if err != nil {
		return fmt.Errorf("failed to get a handle to stdout: %v", err)
	}

	var consoleMode uint32
	err = windows.GetConsoleMode(console, &consoleMode)
	if err != nil {
		return fmt.Errorf("getConsoleMode: %v", err)
	}
	err = windows.SetConsoleMode(console, consoleMode|windows.ENABLE_VIRTUAL_TERMINAL_PROCESSING|windows.ENABLE_PROCESSED_INPUT)
	// err = windows.SetConsoleMode(console, consoleMode|windows.ENABLE_VIRTUAL_TERMINAL_PROCESSING)
	return fmt.Errorf("setConsoleMode: %v", err)
}

func WinIsDataAvailable(handle windows.Handle) (bytesAvailable int, err error) {
	if fPeekNamedPipe.Find() != nil {
		return -1, fmt.Errorf("unsupported version of Windows. PeekNamedPipe not found")
	}
	var numAvail uint32

	ret, _, err := fPeekNamedPipe.Call(uintptr(handle),
		0,
		0,
		0,
		uintptr(unsafe.Pointer(&numAvail)),
		0)
	if ret == 0x0 {
		return -1, err
	}
	return int(numAvail), nil
}

func setRawModOnStdin() (windows.Handle, error) {
	inName, err := windows.UTF16PtrFromString("CONIN$")
	if err != nil {
		return windows.InvalidHandle, err
	}
	var h windows.Handle
	handle, err := windows.CreateFile(
		inName,
		windows.FILE_GENERIC_READ|windows.FILE_GENERIC_WRITE,
		windows.FILE_SHARE_READ|windows.FILE_SHARE_WRITE,
		nil,
		windows.OPEN_EXISTING,
		windows.FILE_ATTRIBUTE_NORMAL,
		h,
	)
	if err != nil {
		return windows.InvalidHandle, err
	}
	var consoleMode uint32
	windows.GetConsoleMode(handle, &consoleMode)
	consoleMode ^= windows.ENABLE_ECHO_INPUT
	consoleMode ^= windows.ENABLE_LINE_INPUT
	consoleMode ^= windows.ENABLE_PROCESSED_INPUT
	consoleMode |= windows.ENABLE_VIRTUAL_TERMINAL_INPUT
	windows.SetConsoleMode(handle, consoleMode)
	return handle, nil
}

func setRawModOnStdout() (windows.Handle, error) {
	inName, err := windows.UTF16PtrFromString("CONOUT$")
	if err != nil {
		return windows.InvalidHandle, err
	}
	var h windows.Handle
	handle, err := windows.CreateFile(
		inName,
		windows.FILE_GENERIC_READ|windows.FILE_GENERIC_WRITE,
		windows.FILE_SHARE_READ|windows.FILE_SHARE_WRITE,
		nil,
		windows.OPEN_EXISTING,
		windows.FILE_ATTRIBUTE_NORMAL,
		h,
	)
	if err != nil {
		return windows.InvalidHandle, err
	}
	var consoleMode uint32
	windows.GetConsoleMode(handle, &consoleMode)
	consoleMode |= windows.ENABLE_PROCESSED_OUTPUT
	consoleMode |= windows.ENABLE_VIRTUAL_TERMINAL_PROCESSING
	windows.SetConsoleMode(handle, consoleMode)
	return handle, nil
}

func SetRawMode() (inHandle, outHandle windows.Handle) {
	inHandle, _ = setRawModOnStdin()
	outHandle, _ = setRawModOnStdout()
	return
}

// CreateEnvBlock converts an array of environment strings into
// the representation required by CreateProcess: a sequence of NUL
// terminated strings followed by a nil.
// Last bytes are two UCS-2 NULs, or four NUL bytes.
// If any string contains a NUL, it returns (nil, EINVAL).
func CreateEnvBlock(envv []string) ([]uint16, error) {
	if len(envv) == 0 {
		return utf16.Encode([]rune("\x00\x00")), nil
	}
	var length int

	for _, s := range envv {
		if strings.IndexByte(s, 0) != -1 {
			return nil, syscall.EINVAL
		}
		length += len(s) + 1
	}
	length += 1

	b := make([]uint16, 0, length)
	for _, s := range envv {
		for _, c := range s {
			b = utf16.AppendRune(b, c)
		}
		b = utf16.AppendRune(b, 0)
	}
	b = utf16.AppendRune(b, 0)
	return b, nil
}
