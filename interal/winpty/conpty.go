//go:build windows

package winpty

import (
	"context"
	"fmt"
	"unsafe"

	"golang.org/x/sys/windows"
)

type HandleWrapper struct {
	handle windows.Handle
}

func (h *HandleWrapper) Read(p []byte) (int, error) {
	var numRead uint32 = 0
	err := windows.ReadFile(h.handle, p, &numRead, nil)
	return int(numRead), err
}

func (h *HandleWrapper) Write(p []byte) (int, error) {
	var numWritten uint32 = 0
	err := windows.WriteFile(h.handle, p, &numWritten, nil)
	return int(numWritten), err
}

func (h *HandleWrapper) Close() error {
	return windows.CloseHandle(h.handle)
}

func (h *HandleWrapper) GetHandle() windows.Handle {
	return h.handle
}

type StartupInfoEx struct {
	startupInfo   windows.StartupInfo
	attributeList []byte
}

const (
	PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE uintptr = 0x20016
	STILL_ACTIVE                        uint32  = 259
)

func GetStartupInfoExForPTY(hpc windows.Handle) (*StartupInfoEx, error) {
	if fInitializeProcThreadAttributeList.Find() != nil {
		return nil, fmt.Errorf("initializeProcThreadAttributeList not found")
	}
	if fUpdateProcThreadAttribute.Find() != nil {
		return nil, fmt.Errorf("updateProcThreadAttribute not found")
	}
	var siEx StartupInfoEx
	siEx.startupInfo.Cb = uint32(unsafe.Sizeof(windows.StartupInfo{}) + unsafe.Sizeof(&siEx.attributeList[0]))
	var size uintptr

	// first call is to get required size. this should return false.
	fInitializeProcThreadAttributeList.Call(0, 1, 0, uintptr(unsafe.Pointer(&size)))
	siEx.attributeList = make([]byte, int(size))
	ret, _, err := fInitializeProcThreadAttributeList.Call(
		uintptr(unsafe.Pointer(&siEx.attributeList[0])),
		1,
		0,
		uintptr(unsafe.Pointer(&size)))
	if ret != 1 {
		return nil, fmt.Errorf("initializeProcThreadAttributeList: %v", err)
	}

	ret, _, err = fUpdateProcThreadAttribute.Call(
		uintptr(unsafe.Pointer(&siEx.attributeList[0])),
		0,
		PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
		uintptr(hpc),
		unsafe.Sizeof(hpc),
		0,
		0)
	if ret != 1 {
		return nil, fmt.Errorf("initializeProcThreadAttributeList: %v", err)
	}
	return &siEx, nil
}

func CreateConsoleProcessAttachedToPTY(hpc windows.Handle, commandLine string) (*windows.ProcessInformation, error) {
	cmdLine, err := windows.UTF16PtrFromString(commandLine)
	if err != nil {
		return nil, err
	}
	siEx, err := GetStartupInfoExForPTY(hpc)
	if err != nil {
		return nil, err
	}
	var pi windows.ProcessInformation

	err = windows.CreateProcess(
		nil, // use this if no args
		cmdLine,
		nil,
		nil,
		false, // inheritHandle
		windows.EXTENDED_STARTUPINFO_PRESENT,
		nil,
		nil,
		&siEx.startupInfo,
		&pi)
	if err != nil {
		return nil, err
	}
	return &pi, nil
}

type ConPty struct {
	hpc    windows.Handle
	pi     *windows.ProcessInformation
	cmdIn  *HandleWrapper
	cmdOut *HandleWrapper
	ptyIn  *HandleWrapper
	ptyOut *HandleWrapper
}

// Wait for the process to exit and return the exit code. If context is canceled,
// Wait() will return STILL_ACTIVE and an error indicating the context was canceled.
func (cpty *ConPty) Wait(ctx context.Context) (uint32, error) {
	var exitCode uint32 = STILL_ACTIVE
	for {
		if err := ctx.Err(); err != nil {
			return STILL_ACTIVE, fmt.Errorf("wait canceled: %v", err)
		}
		ret, _ := windows.WaitForSingleObject(cpty.pi.Process, 1000)
		if ret != uint32(windows.WAIT_TIMEOUT) {
			err := windows.GetExitCodeProcess(cpty.pi.Process, &exitCode)
			return exitCode, err
		}
	}
}

func (cpty *ConPty) Read(p []byte) (int, error) {
	if count, _ := WinIsDataAvailable(cpty.cmdOut.handle); count != 0 {
		return cpty.cmdOut.Read(p)
	}
	return 0, nil
}

func (cpty *ConPty) Write(p []byte) (int, error) {
	return cpty.cmdIn.Write(p)
}

// Close all open handles and terminate the process.
func (cpty *ConPty) Close() error {
	// there is no return code
	Win32ClosePseudoConsole(cpty.hpc)
	return WinCloseHandles(
		cpty.pi.Process,
		cpty.pi.Thread,
		cpty.cmdIn.handle,
		cpty.cmdOut.handle,
		cpty.ptyIn.handle,
		cpty.ptyOut.handle,
	)
}

func (cpty *ConPty) Resize(width, height int) error {
	coords := COORD{
		width,
		height,
	}
	return Win32ResizePseudoConsole(cpty.hpc, &coords)
}

// Start a new process specified in `commandLine` and attach a pseudo console using the Windows
// ConPty API. If ConPty is not available, ErrConPtyUnsupported will be returned.
// On successful return, an instance of ConPty is returned. You must call Close() on this to release
// any resources associated with the process. To get the exit code of the process, you can call Wait().
func Start(commandLine string, coord *COORD) (*ConPty, error) {
	if !WinIsConPtyAvailable() {
		return nil, ErrConPtyUnsupported
	}

	var cmdIn, cmdOut, ptyIn, ptyOut windows.Handle
	if err := windows.CreatePipe(&ptyIn, &cmdIn, nil, 0); err != nil {
		return nil, fmt.Errorf("createPipe: %v", err)
	}
	if err := windows.CreatePipe(&cmdOut, &ptyOut, nil, 0); err != nil {
		WinCloseHandles(ptyIn, cmdIn)
		return nil, fmt.Errorf("createPipe: %v", err)
	}

	inHandle, outHandle := SetRawMode()
	defer WinCloseHandles(
		inHandle,
		outHandle,
	)

	hPc, err := Win32CreatePseudoConsole(coord, ptyIn, ptyOut)
	if err != nil {
		WinCloseHandles(ptyIn, ptyOut, cmdIn, cmdOut)
		return nil, err
	}

	pi, err := CreateConsoleProcessAttachedToPTY(hPc, commandLine)
	if err != nil {
		WinCloseHandles(ptyIn, ptyOut, cmdIn, cmdOut)
		Win32ClosePseudoConsole(hPc)
		return nil, fmt.Errorf("failed to create console process: %v", err)
	}

	cpty := &ConPty{
		hpc:    hPc,
		pi:     pi,
		cmdIn:  &HandleWrapper{cmdIn},
		cmdOut: &HandleWrapper{cmdOut},
		ptyIn:  &HandleWrapper{ptyIn},
		ptyOut: &HandleWrapper{ptyOut},
	}
	return cpty, nil
}
