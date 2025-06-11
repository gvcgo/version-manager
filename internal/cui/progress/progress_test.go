package progress

import (
	"fmt"
	"testing"
)

func TestProgress(t *testing.T) {
	p := NewProgress("test")
	fmt.Println(p.title)
}
