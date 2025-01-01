package self

import (
	"fmt"
	"os"
	"runtime"
	"strings"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/internal/shell"
)

/*
Add custumed source command to shell profile.
Only for unix-like system.
*/

var sourceCmd = `alias svmr="export VM_DISABLE='' && source %s"`

func AddCustomedSourceCmd() {
	if runtime.GOOS == gutils.Windows {
		return
	}
	sheller := shell.NewShell()
	profilePath := sheller.ConfPath()
	content := fmt.Sprintf(sourceCmd, profilePath)

	oldContent, _ := os.ReadFile(profilePath)

	if strings.Contains(string(oldContent), content) {
		return
	}

	newContent := string(oldContent) + "\n" + content
	os.WriteFile(profilePath, []byte(newContent), os.ModePerm)
}
