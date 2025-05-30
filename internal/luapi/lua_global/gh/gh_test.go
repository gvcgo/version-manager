package gh

import (
	"fmt"
	"testing"

	"github.com/gvcgo/version-manager/internal/cnf"
)

func TestGh(t *testing.T) {
	cfg := cnf.NewVMRConf()
	cfg.Load()

	if cfg.GithubToken == "" {
		cfg.GithubToken = GetDefaultReadOnly()
	}

	client := NewGh("oven-sh/bun", cfg.GithubToken, cfg.ProxyUri, cfg.ReverseProxy)
	rl := client.GetReleases()
	if len(rl) == 0 {
		t.Error("test github release failed")
	} else {
		fmt.Printf("release items: %+v\n", rl)
	}
}
