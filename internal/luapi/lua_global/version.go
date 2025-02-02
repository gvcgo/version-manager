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

func NewVersionList(L *lua.LState) int {
	ud := L.NewUserData()
	ud.Value = make(VersionList)
	L.Push(ud)
	return 1
}

func AddItem(L *lua.LState) int {
	ud := L.ToUserData(1)
	if ud == nil {
		return 0
	}
	vl, ok := ud.Value.(VersionList)

	if !ok || vl == nil {
		return 0
	}

	versionStr := L.ToString(2)
	if versionStr == "" {
		return 0
	}

	vSdk, ok2 := vl[versionStr]
	if !ok2 {
		vl[versionStr] = make(SDKVersion, 0)
	}

	itemTable := L.ToTable(3)
	if itemTable == nil {
		return 0
	}

	item := Item{
		Url:       itemTable.RawGetString("url").String(),
		Arch:      itemTable.RawGetString("arch").String(),
		Os:        itemTable.RawGetString("os").String(),
		Sum:       itemTable.RawGetString("sum").String(),
		SumType:   itemTable.RawGetString("sum_type").String(),
		Size:      gconv.Int64(itemTable.RawGetString("size").String()),
		Installer: itemTable.RawGetString("installer").String(),
		LTS:       itemTable.RawGetString("lts").String(),
		Extra:     itemTable.RawGetString("extra").String(),
	}

	vSdk = append(vSdk, item)
	vl[versionStr] = vSdk

	result := L.NewUserData()
	result.Value = item
	L.Push(result)
	return 1
}
