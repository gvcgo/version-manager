package utils

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gtea/gprint"
	"github.com/gvcgo/goutils/pkgs/gutils"

	Arch "github.com/gvcgo/goutils/pkgs/archiver"
)

func Untar(srcPath, dstDir string) (err error) {
	_, err = gutils.ExecuteSysCommand(
		true,
		"",
		"tar",
		"-xf",
		srcPath,
		"-C",
		dstDir,
	)
	return
}

func Unzip(srcPath, dstDir string) (err error) {
	// use unzip command in mingw bash.
	if runtime.GOOS == gutils.Windows && !IsMingWBash() {
		// expand -r file.zip C:\Users\username\Desktop\extracted
		pathString := os.Getenv("PATH")
		newPathString := fmt.Sprintf("%s;%s", `C:\Windows\System32`, pathString)
		os.Setenv("PATH", newPathString)
		_, err = gutils.ExecuteSysCommand(true, "",
			"powershell",
			"expand",
			"-r",
			srcPath,
			dstDir)
	} else {
		// unzip file.zip -d extracted
		_, err = gutils.ExecuteSysCommand(true, "",
			"unzip",
			srcPath,
			"-d",
			dstDir)
	}
	return
}

func DecompressBySystemCommand(srcPath, dstDir string) (err error) {
	if strings.HasSuffix(srcPath, ".zip") {
		err = Unzip(srcPath, dstDir)
	} else if strings.HasSuffix(srcPath, ".tar") || strings.Contains(srcPath, ".tar.") {
		err = Untar(srcPath, dstDir)
	} else {
		err = fmt.Errorf("unsupported by system command")
	}
	return
}

// use archiver or xtract.
func UseArchiver(srcPath string) bool {
	if strings.HasSuffix(srcPath, ".gz") && !strings.HasSuffix(srcPath, ".tar.gz") {
		return false
	}
	if strings.HasSuffix(srcPath, ".7z") {
		return false
	}
	if strings.Contains(strings.ToLower(srcPath), "odin") {
		return false
	}
	return true
}

func handleMultiCompress(destDir string) {
	// for odin and clojure.
	tempDirList, _ := os.ReadDir(destDir)
	for _, d := range tempDirList {
		dd := filepath.Join(destDir, d.Name())
		if !d.IsDir() && strings.HasSuffix(d.Name(), ".zip") {
			aa, _ := Arch.NewArchiver(dd, destDir, true)
			aa.UnArchive()
		}
	}
}

/*
Decompress archived files.
*/
func Extract(srcFile, destDir string) (err error) {
	if ok, _ := gutils.PathIsExist(destDir); !ok {
		os.MkdirAll(destDir, os.ModePerm)
	}

	gprint.PrintInfo("Extracting files, please wait...")

	// try to use system unzip or tar.
	if err = DecompressBySystemCommand(srcFile, destDir); err == nil {
		handleMultiCompress(destDir)
		return err
	}

	if arch, err1 := Arch.NewArchiver(srcFile, destDir, UseArchiver(srcFile)); err1 == nil {
		_, err = arch.UnArchive()
		if err != nil {
			return
		}
		handleMultiCompress(destDir)
		return
	} else {
		err = err1
	}
	return
}
