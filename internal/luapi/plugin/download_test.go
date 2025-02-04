package plugin

import (
	"fmt"
	"testing"
)

func TestDownload(t *testing.T) {
	fmt.Println("test download aaaaa")
	if err := downloadPlugins(); err != nil {
		t.Error(err)
	}
}

func TestUpdatePlugins(t *testing.T) {
	fmt.Println("test update plugins")
	if err := UpdatePlugins(); err != nil {
		t.Error(err)
	}
}
