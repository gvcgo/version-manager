package request

import (
	"testing"
)

func TestReqClient(t *testing.T) {
	// dURL := "https://go.dev/dl/go1.10.linux-amd64.tar.gz"
	rc := New()
	if rc.Client == nil {
		t.Error("client is nil!")
	}
	// file, err := os.Create("/home/moqsien/myprojects/go/version-manager/internal/cui/progress/go1.10.linux-amd64.tar.gz")
	// if err != nil {
	// 	t.Error(err)
	// }
	// defer file.Close()
	// _, err = rc.DoDownloadToWriter(file, dURL)
	// if err != nil {
	// 	t.Error(err)
	// }
}
