package utils

import (
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/gogf/gf/v2/util/gconv"
)

func CopyFile(src, dst string) (written int64, err error) {
	srcFile, err := os.Open(src)

	if err != nil {
		return 0, err
	}
	defer srcFile.Close()

	dstFile, err := os.OpenFile(dst, os.O_WRONLY|os.O_CREATE, os.ModePerm)
	if err != nil {
		return 0, fmt.Errorf("open dst file failed: %s", err.Error())
	}
	defer dstFile.Close()

	return io.Copy(dstFile, srcFile)
}

// CopyAFile copies the file at the source path to the provided destination.
func CopyAFile(source, destination string) error {
	//Validate the source and destination paths
	if len(source) == 0 {
		return errors.New("no source file path provided")
	}

	if len(destination) == 0 {
		return errors.New("no destination file path provided")
	}

	//Verify the source path refers to a regular file
	sourceFileInfo, err := os.Lstat(source)
	if err != nil {
		return err
	}

	//Handle regular files differently than symbolic links and other non-regular files.
	if sourceFileInfo.Mode().IsRegular() {
		//open the source file
		sourceFile, err := os.Open(source)
		if err != nil {
			return err
		}
		defer sourceFile.Close()

		//create the destinatin file
		destinationFile, err := os.Create(destination)
		if err != nil {
			return err
		}
		defer destinationFile.Close()

		//copy the source file contents to the destination file
		if _, err = io.Copy(destinationFile, sourceFile); err != nil {
			return err
		}

		//replicate the source file mode for the destination file
		if err := os.Chmod(destination, sourceFileInfo.Mode()); err != nil {
			return err
		}
	} else if sourceFileInfo.Mode()&os.ModeSymlink != 0 {
		linkDestinaton, err := os.Readlink(source)
		if err != nil {
			return errors.New("Unable to read symlink. " + err.Error())
		}

		if err := os.Symlink(linkDestinaton, destination); err != nil {
			return errors.New("Unable to replicate symlink. " + err.Error())
		}
	} else {
		return errors.New("Unable to use io.Copy on file with mode " + gconv.String(sourceFileInfo.Mode()))
	}

	return nil
}

// CopyDirectory copies the directory at the source path to the provided destination, recursively copying subdirectories.
func CopyDirectory(source string, destination string) error {
	if len(source) == 0 || len(destination) == 0 {
		return errors.New("file paths must not be empty")
	}

	//get properties of the source directory
	sourceInfo, err := os.Stat(source)
	if err != nil {
		return err
	}

	//create the destination directory
	err = os.MkdirAll(destination, sourceInfo.Mode())
	if err != nil {
		return err
	}

	sourceDirectory, err := os.Open(source)
	if err != nil {
		return err
	}
	defer sourceDirectory.Close()

	objects, err := sourceDirectory.Readdir(-1)
	if err != nil {
		return err
	}

	for _, object := range objects {
		if object.Name() == ".Trashes" || object.Name() == ".DS_Store" {
			continue
		}

		sourceObjectName := source + string(filepath.Separator) + object.Name()
		destObjectName := destination + string(filepath.Separator) + object.Name()

		if object.IsDir() {
			//create sub-directories
			err = CopyDirectory(sourceObjectName, destObjectName)
			if err != nil {
				return err
			}
		} else {
			err = CopyAFile(sourceObjectName, destObjectName)
			if err != nil {
				return err
			}
		}
	}

	return nil
}
