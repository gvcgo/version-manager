package lua_global

import (
	"math"
	"regexp"
	"runtime"
	"sort"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gogf/gf/v2/util/gutil"
	lua "github.com/yuin/gopher-lua"
)

/*
Parse version number.
*/
var (
	versionRegexp = regexp.MustCompile(`\d+(.\d+){0,2}`)
	betaRegexp    = regexp.MustCompile(`beta\.*\d+`)
	rcRegexp      = regexp.MustCompile(`rc\.*\d+`)
	numRegexp     = regexp.MustCompile(`\d+`)
)

// Version represents a version number.
type Version struct {
	RawVersion string
	Major      int
	Minor      int
	Patch      int
	Build      int
	Beta       int
	RC         int
	ok         bool
	parsed     bool
}

func NewVersion(rawVersion string) *Version {
	return &Version{
		RawVersion: rawVersion,
	}
}

func (v *Version) IsOk() bool {
	return v.ok
}

func (v *Version) IsParsed() bool {
	return v.parsed
}

// ParseVersion parses a version string into a Version struct.
func (v *Version) Parse() {
	if v.parsed {
		return
	}

	defer func() {
		v.parsed = true
	}()

	version := strings.ToLower(v.RawVersion)
	vstr := versionRegexp.FindString(version)
	bstr := betaRegexp.FindString(version)
	rstr := rcRegexp.FindString(version)

	if vstr == "" {
		v.ok = false
		return
	}
	parts := strings.Split(vstr, ".")

	for i, part := range parts {
		switch i {
		case 0:
			v.Major = gconv.Int(part)
		case 1:
			v.Minor = gconv.Int(part)
		case 2:
			v.Patch = gconv.Int(part)
		case 3:
			v.Build = gconv.Int(part)
		default:
		}
	}

	v.Beta = gconv.Int(numRegexp.FindString(bstr))
	v.RC = gconv.Int(numRegexp.FindString(rstr))

	if v.Beta == 0 && !strings.Contains(version, "beta") {
		v.Beta = math.MaxInt
	} else if v.Beta == 0 && strings.Contains(version, "beta") {
		v.Beta = 1
	}

	if v.RC == 0 && !strings.Contains(version, "rc") {
		v.RC = math.MaxInt
	} else if v.RC == 0 && strings.Contains(version, "rc") {
		v.RC = 1
	}
	v.ok = true
	return
}

func (v *Version) IsGreaterThan(other *Version) bool {
	if !v.IsParsed() {
		v.Parse()
	}
	if !other.IsParsed() {
		other.Parse()
	}

	if !v.IsOk() || !other.IsOk() {
		return gutil.ComparatorString(v.RawVersion, other.RawVersion) >= 0
	}

	if v.Major != other.Major {
		return v.Major > other.Major
	}

	if v.Minor != other.Minor {
		return v.Minor > other.Minor
	}

	if v.Patch != other.Patch {
		return v.Patch > other.Patch
	}

	if v.Build != other.Build {
		return v.Build > other.Build
	}

	if v.Beta != other.Beta {
		return v.Beta > other.Beta
	}
	return v.RC > other.RC
}

/*
Downloaded version item.
*/
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
	Url       string `json:"url"`        // download url
	Arch      string `json:"arch"`       // amd64 | arm64
	Os        string `json:"os"`         // linux | darwin | windows
	Sum       string `json:"sum"`        // Checksum
	SumType   string `json:"sum_type"`   // sha1 | sha256 | sha512 | md5
	Size      int64  `json:"size"`       // Size in bytes
	Installer string `json:"installer"`  // conda | conda-forge | coursier | unarchiver | executable | dpkg | rpm
	LTS       string `json:"lts"`        // Long Term Support
	Extra     string `json:"extra"`      // Extra Info
	CreatedAt int64  `json:"created_at"` // Unix timestamp
}

// type SDKVersion []Item

// type VersionList map[string]SDKVersion
type VersionList map[string]Item

func (vl VersionList) SortDesc() (verList []string) {
	if len(vl) == 0 {
		return
	}
	for k := range vl {
		verList = append(verList, k)
	}

	hasCreatedAt := false
	for _, item := range vl {
		if item.CreatedAt > 0 {
			hasCreatedAt = true
			break
		}
	}

	if hasCreatedAt {
		sort.Slice(verList, func(i, j int) bool {
			ver1 := vl[verList[i]]
			ver2 := vl[verList[j]]
			return ver1.CreatedAt > ver2.CreatedAt
		})
	} else {
		sort.Slice(verList, func(i, j int) bool {
			ver1 := NewVersion(verList[i])
			ver2 := NewVersion(verList[j])
			return ver1.IsGreaterThan(ver2)
		})
	}
	return
}

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

	itemTable := L.ToTable(3)
	if itemTable == nil {
		result := L.NewUserData()
		result.Value = vl
		L.Push(result)
		return 1
	}

	archInfo := GetStringFromLTable(itemTable, "arch")
	osInfo := GetStringFromLTable(itemTable, "os")
	if archInfo != runtime.GOARCH || osInfo != runtime.GOOS {
		result := L.NewUserData()
		result.Value = vl
		L.Push(result)
		return 1
	}

	item := Item{}
	item.Url = GetStringFromLTable(itemTable, "url")
	item.Arch = archInfo
	item.Os = osInfo
	item.Sum = GetStringFromLTable(itemTable, "sum")
	item.SumType = GetStringFromLTable(itemTable, "sum_type")
	item.Size = gconv.Int64(GetStringFromLTable(itemTable, "size"))
	item.Installer = GetStringFromLTable(itemTable, "installer")
	item.LTS = GetStringFromLTable(itemTable, "lts")
	item.Extra = GetStringFromLTable(itemTable, "extra")
	vl[versionStr] = item

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
		// sdkVersion, ok := vl[k]
		// if !ok {
		// 	vl[k] = v
		// } else {
		// 	vl[k] = append(sdkVersion, v...)
		// }
		vl[k] = v
	}

	result := L.NewUserData()
	result.Value = vl
	L.Push(result)
	return 1
}
