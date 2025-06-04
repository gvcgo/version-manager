package lua_global

import (
	"github.com/gogf/gf/v2/util/gconv"
	lua "github.com/yuin/gopher-lua"
)

const (
	Conda      string = "conda"
	CondaForge string = "conda-forge"
	Coursier   string = "coursier"
	Unarchiver string = "unarchiver"
	Executable string = "executable"
	Dpkg       string = "dpkg"
	Rpm        string = "rpm"
)

type Item struct {
	Url       string `json:"url"`       // download url
	Arch      string `json:"arch"`      // amd64 | arm64
	Os        string `json:"os"`        // linux | darwin | windows
	Sum       string `json:"sum"`       // Checksum
	SumType   string `json:"sum_type"`  // sha1 | sha256 | sha512 | md5
	Size      int64  `json:"size"`      // Size in bytes
	Installer string `json:"installer"` // conda | conda-forge | coursier | unarchiver | executable | dpkg | rpm
	LTS       string `json:"lts"`       // Long Term Support
	Extra     string `json:"extra"`     // Extra Info
}

type SDKVersion []Item

type VersionList map[string]SDKVersion

/*
lua: vl = vmrNewVersionList()
*/
func NewVersionList(L *lua.LState) int {
	ud := L.NewUserData()
	ud.Value = make(VersionList)
	L.Push(ud)
	return 1
}

func GetStringFromLTable(table *lua.LTable, key string) string {
	if table == nil {
		return ""
	}
	value := table.RawGetString(key).String()
	if value == "nil" {
		return ""
	}
	return value
}

/*
lua:
vl = vmrNewVersionList()
item = { ["url"] = "xxx", ["arch"] = "xxx", ["os"] = "xxx" }
vmrAddItem(vl, versionName, item)
*/
func AddItem(L *lua.LState) int {

	ud := L.ToUserData(1)
	if ud == nil {
		result := L.NewUserData()
		result.Value = nil
		L.Push(result)
		return 1
	}
	vl, ok := ud.Value.(VersionList)

	if !ok || vl == nil {
		result := L.NewUserData()
		result.Value = nil
		L.Push(result)
		return 1
	}

	versionStr := L.ToString(2)
	if versionStr == "" {
		result := L.NewUserData()
		result.Value = vl
		L.Push(result)
		return 1
	}

	vSdk, ok2 := vl[versionStr]
	if !ok2 {
		vl[versionStr] = make(SDKVersion, 0)
	}

	itemTable := L.ToTable(3)
	if itemTable == nil {
		result := L.NewUserData()
		result.Value = vl
		L.Push(result)
		return 1
	}

	item := Item{}
	item.Url = GetStringFromLTable(itemTable, "url")
	item.Arch = GetStringFromLTable(itemTable, "arch")
	item.Os = GetStringFromLTable(itemTable, "os")
	item.Sum = GetStringFromLTable(itemTable, "sum")
	item.SumType = GetStringFromLTable(itemTable, "sum_type")
	item.Size = gconv.Int64(GetStringFromLTable(itemTable, "size"))
	item.Installer = GetStringFromLTable(itemTable, "installer")
	item.LTS = GetStringFromLTable(itemTable, "lts")
	item.Extra = GetStringFromLTable(itemTable, "extra")

	vSdk = append(vSdk, item)
	vl[versionStr] = vSdk

	result := L.NewUserData()
	result.Value = vl
	L.Push(result)
	return 1
}

/*
lua:
vl1 = vmrNewVersionList()
vl2 = vmrNewVersionList()
vl = vmrMergeVersionList(vl1, vl2)
*/
func MergeVersionList(L *lua.LState) int {
	ud := L.ToUserData(1)
	if ud == nil {
		result := L.NewUserData()
		result.Value = nil
		L.Push(result)
		return 1
	}
	vl, ok := ud.Value.(VersionList)

	if !ok || vl == nil {
		result := L.NewUserData()
		result.Value = nil
		L.Push(result)
		return 1
	}

	ud2 := L.ToUserData(2)
	if ud2 == nil {
		result := L.NewUserData()
		result.Value = vl
		L.Push(result)
		return 1
	}
	vl2, ok2 := ud2.Value.(VersionList)

	if !ok2 || vl2 == nil {
		result := L.NewUserData()
		result.Value = vl
		L.Push(result)
		return 1
	}

	for k, v := range vl2 {
		sdkVersion, ok3 := vl[k]
		if !ok3 {
			vl[k] = v
		} else {
			vl[k] = append(sdkVersion, v...)
		}
	}

	result := L.NewUserData()
	result.Value = vl
	L.Push(result)
	return 1
}
