package lua_global

import lua "github.com/yuin/gopher-lua"

/*
Installation configurations.
*/

type FileItems struct {
	Windows []string `json:"windows"`
	Linux   []string `json:"linux"`
	MacOS   []string `json:"darwin"`
}

type (
	DirPath  []string
	DirItems struct {
		Windows []DirPath `json:"windows"` // <symbolLinkPath>/<filepath.Join(List)>, ...
		Linux   []DirPath `json:"linux"`
		MacOS   []DirPath `json:"darwin"`
	}
)

type AdditionalEnv struct {
	Name    string    `json:"name"`
	Value   []DirPath `json:"value"`   // <symbolLinkPath>/<filepath.Join(Value)>
	Version string    `json:"version"` // major>8 or major<=8(for JDK)
}

type AdditionalEnvList []AdditionalEnv

type BinaryRename struct {
	NameFlag string `json:"name_flag"`
	RenameTo string `json:"rename_to"`
}

/*
Installation configs
*/
type InstallerConfig struct {
	FlagFiles       *FileItems        `json:"flag_files"`
	FlagDirExcepted bool              `json:"flag_dir_excepted"`
	BinaryDirs      *DirItems         `json:"binary_dirs"`
	BinaryRename    *BinaryRename     `json:"binary_rename"`
	AdditionalEnvs  AdditionalEnvList `json:"additional_envs"`
}

func NewInstallerConfig() (ic *InstallerConfig) {
	ic = &InstallerConfig{
		FlagFiles:      &FileItems{},
		BinaryDirs:     &DirItems{},
		BinaryRename:   &BinaryRename{},
		AdditionalEnvs: AdditionalEnvList{},
	}
	return
}

func NewInstallerConf(L *lua.LState) int {
	ud := L.NewUserData()
	ud.Value = NewInstallerConfig()
	L.Push(ud)
	return 1
}

func checkInstallerConfig(L *lua.LState, n int) *InstallerConfig {
	ud := L.ToUserData(n)
	if ic, ok := ud.Value.(*InstallerConfig); ok {
		return ic
	}
	return nil
}

func AddFlagFiles(L *lua.LState) int {
	ic := checkInstallerConfig(L, 1)
	if ic == nil {
		return 0
	}
	osStr := L.ToString(2)

	value := []string{}
	vList := L.ToTable(3)
	vList.ForEach(func(l1, l2 lua.LValue) {
		value = append(value, l2.String())
	})

	if osStr == "" {
		ic.FlagFiles.Windows = append(ic.FlagFiles.Windows, value...)
		ic.FlagFiles.Linux = append(ic.FlagFiles.Linux, value...)
		ic.FlagFiles.MacOS = append(ic.FlagFiles.MacOS, value...)
	} else {
		switch osStr {
		case "windows":
			ic.FlagFiles.Windows = append(ic.FlagFiles.Windows, value...)
		case "linux":
			ic.FlagFiles.Linux = append(ic.FlagFiles.Linux, value...)
		case "darwin":
			ic.FlagFiles.MacOS = append(ic.FlagFiles.MacOS, value...)
		default:
			return 0
		}
	}

	result := L.NewUserData()
	result.Value = ic
	L.Push(result)
	return 1
}

func EnableFlagDirExcepted(L *lua.LState) int {
	ic := checkInstallerConfig(L, 1)
	if ic == nil {
		return 0
	}
	ic.FlagDirExcepted = true

	result := L.NewUserData()
	result.Value = ic
	L.Push(result)
	return 1
}

func AddBinaryDirs(L *lua.LState) int {
	ic := checkInstallerConfig(L, 1)
	if ic == nil {
		return 0
	}

	osStr := L.ToString(2)

	value := []string{}
	vList := L.ToTable(3)
	vList.ForEach(func(l1, l2 lua.LValue) {
		value = append(value, l2.String())
	})

	if osStr == "" {
		ic.BinaryDirs.Windows = append(ic.BinaryDirs.Windows, value)
		ic.BinaryDirs.Linux = append(ic.BinaryDirs.Linux, value)
		ic.BinaryDirs.MacOS = append(ic.BinaryDirs.MacOS, value)
	} else {
		switch osStr {
		case "windows":
			ic.BinaryDirs.Windows = append(ic.BinaryDirs.Windows, value)
		case "linux":
			ic.BinaryDirs.Linux = append(ic.BinaryDirs.Linux, value)
		case "darwin":
			ic.BinaryDirs.MacOS = append(ic.BinaryDirs.MacOS, value)
		default:
			return 0
		}
	}

	result := L.NewUserData()
	result.Value = ic
	L.Push(result)
	return 1
}

func AddAdditionalEnvs(L *lua.LState) int {
	ic := checkInstallerConfig(L, 1)
	if ic == nil {
		return 0
	}

	envName := L.ToString(2)
	envPath := L.ToTable(3)
	version := L.ToString(4)

	if envName == "" || envPath == nil {
		return 0
	}

	value := AdditionalEnv{
		Name:    envName,
		Version: version,
	}

	pathValue := DirPath{}
	envPath.ForEach(func(l1, l2 lua.LValue) {
		pathValue = append(pathValue, l2.String())
	})
	value.Value = append(value.Value, pathValue)
	ic.AdditionalEnvs = append(ic.AdditionalEnvs, value)

	result := L.NewUserData()
	result.Value = ic
	L.Push(result)
	return 1
}

const (
	InstallerConfigName string = "ic"
)

func GetInstallerConfig(L *lua.LState) *InstallerConfig {
	v := L.GetGlobal(InstallerConfigName)

	if v.Type() == lua.LTUserData {
		ud := v.(*lua.LUserData)
		if ud == nil {
			return nil
		}
		if ic, ok := ud.Value.(*InstallerConfig); ok {
			return ic
		}
	}
	return nil
}
