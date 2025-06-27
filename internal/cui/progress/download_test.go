package progress

import (
	"testing"
)

func TestDownloader(t *testing.T) {
	dURL := "https://go.dev/dl/go1.10.linux-amd64.tar.gz"
	d := NewDownloader(dURL)
	d.SetOutputFilePath("/home/moqsien/myprojects/go/version-manager/internal/cui/progress/go1.10.linux-amd64.tar.gz")
	if d.outputFile == "" {
		t.Error("outputFile is not set.")
	}
	// pro := tea.NewProgram(d)

	// d.AddOptions(
	// 	WithProgram(pro),
	// 	WithTotal(d.GetTotalSize()),
	// )

	// if _, err := pro.Run(); err != nil {
	// 	t.Errorf("failed to start program: %v", err)
	// }
}

func TestDownloaderMulti(t *testing.T) {
	dURL := "https://go.dev/dl/go1.10.linux-amd64.tar.gz"
	d := NewDownloader(dURL)
	d.SetThreads(3)
	d.SetOutputFilePath("/home/moqsien/myprojects/go/version-manager/internal/cui/progress/go1.10.linux-amd64.tar.gz")
	if d.outputFile == "" {
		t.Error("outputFile is not set.")
	}
	// pro := tea.NewProgram(d)

	// d.AddOptions(
	// 	WithProgram(pro),
	// 	WithTotal(d.GetTotalSize()),
	// )

	// if _, err := pro.Run(); err != nil {
	// 	t.Errorf("failed to start program: %v", err)
	// }
}
