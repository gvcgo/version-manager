package versions

import (
	"strconv"
	"strings"

	"github.com/gogf/gf/v2/util/gutil"
)

/*
General QuickSort
*/
type Item interface {
	Greater(Item) bool
	String() string
}

var isAscend bool

func partition(iList []Item, left int, right int) (mid int) {
	pivot := iList[right]
	i := left
	j := left
	for j < right {
		if isAscend {
			if !iList[j].Greater(pivot) {
				iList[i], iList[j] = iList[j], iList[i]
				i++
			}
		} else {
			if iList[j].Greater(pivot) {
				iList[i], iList[j] = iList[j], iList[i]
				i++
			}
		}
		j++
	}
	iList[i], iList[right] = iList[right], iList[i]
	return i
}

func quickSort(iList []Item, left int, right int) {
	if left < right {
		mid := partition(iList, left, right)
		quickSort(iList, left, mid-1)
		quickSort(iList, mid+1, right)
	}
}

func QuickSort(iList []Item, ascend bool) (r []string) {
	isAscend = ascend
	quickSort(iList, 0, len(iList)-1)
	for _, itm := range iList {
		r = append(r, itm.String())
	}
	return
}

/*
Version Comparator
*/
type VersionComparator struct {
	Major  int
	Minor  int
	Patch  int
	Beta   int
	RC     int
	Origin string
}

func (ver *VersionComparator) Greater(item Item) bool {
	v, ok := item.(*VersionComparator)
	if !ok {
		panic("unknown item")
	}
	if ver.Major > v.Major {
		return true
	}
	if ver.Major < v.Major {
		return false
	}
	if ver.Minor > v.Minor {
		return true
	}
	if ver.Minor < v.Minor {
		return false
	}
	if ver.Patch > v.Patch {
		return true
	}
	if ver.Patch < v.Patch {
		return false
	}
	if ver.RC != v.RC {
		if (ver.RC > v.RC && v.RC != 0) || (ver.RC == 0 && ver.Beta == 0) {
			return true
		}
	}
	if ver.Beta != v.Beta {
		if (ver.Beta > v.Beta && v.Beta != 0) || ver.Beta == 0 {
			return true
		}
	}
	return false
}

func (ver *VersionComparator) String() string {
	return ver.Origin
}

// Sorts version list
func SortVersion(vs []string) []string {
	vList := []Item{}
	var vresult []string
	m := make(map[string]struct{}, 50)
	for _, v := range vs {
		if _, ok := m[v]; ok {
			continue
		}
		m[v] = struct{}{}
		vs_ := VersionComparator{}
		if strings.Contains(v, "beta") {
			result := strings.Split(v, "beta")
			vresult = strings.Split(result[0], ".")
			vs_.Beta, _ = strconv.Atoi(result[1])

		} else if strings.Contains(v, "rc") {
			result := strings.Split(v, "rc")
			vresult = strings.Split(result[0], ".")
			vs_.RC, _ = strconv.Atoi(result[1])
		} else {
			vresult = strings.Split(v, ".")

		}
		vs_.Major, _ = strconv.Atoi(vresult[0])
		switch len(vresult) {
		case 2:
			vs_.Minor, _ = strconv.Atoi(vresult[1])
		case 3:
			vs_.Minor, _ = strconv.Atoi(vresult[1])
			vs_.Patch, _ = strconv.Atoi(vresult[2])
		}
		vs_.Origin = v
		vList = append(vList, &vs_)
	}
	return QuickSort(vList, false)
}

type StringComparator struct {
	Origin string
}

func (sc *StringComparator) Greater(item Item) bool {
	r := gutil.ComparatorString(sc.Origin, item.String())
	return r >= 0
}

func (sc *StringComparator) String() string {
	return sc.Origin
}

func SortStringList(sl []string) []string {
	iList := []Item{}
	for _, s := range sl {
		iList = append(iList, &StringComparator{s})
	}
	return QuickSort(iList, true)
}
