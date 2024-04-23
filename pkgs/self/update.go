package self

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/gvcgo/goutils/pkgs/gutils"
	"github.com/gvcgo/version-manager/pkgs/conf"
)

const (
	updateScriptName string = "vmr-update"
)

var (
	WinScript  string = `powershell -nop -c "iex(New-Object Net.WebClient).DownloadString('https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.ps1')"`
	UnixScript string = `curl --proto '=https' --tlsv1.2 -sSf https://gvc.1710717.xyz/proxy/https://raw.githubusercontent.com/gvcgo/version-manager/main/scripts/install.sh | sh`
)

func setUpdateForWindows() {
	scriptPath := filepath.Join(conf.GetManagerDir(), fmt.Sprintf("%s.bat", updateScriptName))
	os.WriteFile(scriptPath, []byte(WinScript), os.ModePerm)
}

func setUpdateForUnix() {
	scriptPath := filepath.Join(conf.GetManagerDir(), updateScriptName)
	os.WriteFile(scriptPath, []byte(UnixScript), os.ModePerm)
}

func SetUpdateScript() {
	if runtime.GOOS == gutils.Windows {
		setUpdateForWindows()
	} else {
		setUpdateForUnix()
	}
}
