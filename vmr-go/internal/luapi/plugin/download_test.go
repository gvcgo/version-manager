package plugin

import (
	"testing"
)

// func TestDownload(t *testing.T) {
// 	fmt.Println("test download aaaaa")
// 	if err := downloadPlugins(); err != nil {
// 		t.Error(err)
// 	}
// }

// func TestUpdatePlugins(t *testing.T) {
// 	fmt.Println("test update plugins")
// 	if err := UpdatePlugins(); err != nil {
// 		t.Error(err)
// 	}
// }

func TestGetPluginFileList(t *testing.T) {
	fileList := GetPluginFileList()
	if len(fileList) == 0 {
		t.Error("no plugin files found")
	}
}
