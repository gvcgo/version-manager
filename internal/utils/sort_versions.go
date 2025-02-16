package utils

import (
	"fmt"
	"math"
	"regexp"
	"sort"
	"strings"

	"github.com/gogf/gf/v2/util/gconv"
	"github.com/gogf/gf/v2/util/gutil"
	"github.com/gvcgo/version-manager/internal/tui/table"
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
	Build int
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
	return
}

// SortVersions sorts a slice of version strings in descending order.
func SortVersions(versions []table.Row) {
	sort.Slice(versions, func(i, j int) bool {
		v1, err := ParseVersion(versions[i][0])
		if err != nil {
			return gutil.ComparatorString(versions[i], versions[j]) >= 0
		}
		v2, err := ParseVersion(versions[j][0])
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

		if v1.Build != v2.Build {
			return v1.Build > v2.Build
		}

		if v1.Beta != v2.Beta {
			return v1.Beta > v2.Beta
		}
		return v1.RC > v2.RC
	})
}

func SortVersionAscend(versions []table.Row) {
	sort.Slice(versions, func(i, j int) bool {
		v1, err := ParseVersion(versions[i][0])
		if err != nil {
			return gutil.ComparatorString(versions[i], versions[j]) < 0
		}
		v2, err := ParseVersion(versions[j][0])
		if err != nil {
			return gutil.ComparatorString(versions[i], versions[j]) < 0
		}
		if v1.Major != v2.Major {
			return v1.Major < v2.Major
		}

		if v1.Minor != v2.Minor {
			return v1.Minor < v2.Minor
		}

		if v1.Patch != v2.Patch {
			return v1.Patch < v2.Patch
		}

		if v1.Build != v2.Build {
			return v1.Build < v2.Build
		}

		if v1.Beta != v2.Beta {
			return v1.Beta < v2.Beta
		}
		return v1.RC < v2.RC
	})
}
