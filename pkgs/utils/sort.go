package utils

import (
	"fmt"
	"regexp"
	"sort"
	"strconv"
	"strings"

	"github.com/gogf/gf/v2/util/gutil"
)

/*
Version Sorter
*/

var (
	versionRegexp = regexp.MustCompile(`\d+(.\d+){0,2}`)
	betaRegexp    = regexp.MustCompile(`beta\.*\d+`)
	rcRegexp      = regexp.MustCompile(`rc\.*\d+`)
	numRegexp     = regexp.MustCompile(`\d+`)
)

// Version represents a version number.
type Version struct {
	Major int
	Minor int
	Patch int
	Beta  int
	RC    int
}

// ParseVersion parses a version string into a Version struct.
func ParseVersion(version string) (v Version, err error) {
	version = strings.ToLower(version)
	vstr := versionRegexp.FindString(version)
	bstr := betaRegexp.FindString(version)
	rstr := rcRegexp.FindString(version)

	v = Version{}
	if vstr == "" {
		return v, fmt.Errorf("can not parse: %s", version)
	}
	parts := strings.Split(vstr, ".")

	for i, part := range parts {
		switch i {
		case 0:
			v.Major, _ = strconv.Atoi(part)
		case 1:
			v.Minor, _ = strconv.Atoi(part)
		case 2:
			v.Patch, _ = strconv.Atoi(part)
		default:
		}
	}

	if bstr != "" {
		v.Beta, _ = strconv.Atoi(numRegexp.FindString(bstr))
	}
	if rstr != "" {
		v.RC, _ = strconv.Atoi(numRegexp.FindString(rstr))
	}
	return
}

// SortVersions sorts a slice of version strings in descending order.
func SortVersions(versions []string) {
	sort.Slice(versions, func(i, j int) bool {
		v1, err := ParseVersion(versions[i])
		if err != nil {
			return gutil.ComparatorString(versions[i], versions[j]) >= 0
		}
		v2, err := ParseVersion(versions[j])
		if err != nil {
			return gutil.ComparatorString(versions[i], versions[j]) >= 0
		}
		if v1.Major != v2.Major {
			return v1.Major > v2.Major
		}

		if v1.Minor != v2.Minor {
			return v1.Minor > v2.Minor
		}

		if v1.Patch != v2.Patch {
			return v1.Patch > v2.Patch
		}

		if v1.Beta != v2.Beta {
			return v1.Beta > v2.Beta
		}
		return v1.RC > v2.RC
	})
}
