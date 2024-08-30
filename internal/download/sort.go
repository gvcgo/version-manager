package download

import (
	"fmt"
	"math"
	"regexp"
	"sort"
	"strconv"
	"strings"

	"github.com/gogf/gf/v2/util/gutil"
	"github.com/gvcgo/version-manager/internal/tui/table"
)

/*
Version Sorter
*/

var (
	prefixRegexp = regexp.MustCompile(`^(\D+)(.*)$`)
	digitsRegexp = regexp.MustCompile(`^(\d+)(.*)$`)
	dotRegexp    = regexp.MustCompile(`^\.(.*)$`)
	betaRegexp   = regexp.MustCompile(`(?i)beta\.*(\d+)`)
	rcRegexp     = regexp.MustCompile(`(?i)rc\.*(\d+)`)
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
	v = Version{}
	parts, suffix, err := parseVersion(version)
	if err != nil {
		return
	}
	v.Major, _ = strconv.Atoi(parts[0])
	v.Minor, _ = strconv.Atoi(parts[1])
	v.Patch, _ = strconv.Atoi(parts[2])
	parts = betaRegexp.FindStringSubmatch(suffix)
	if len(parts) > 1 {
		v.Beta, _ = strconv.Atoi(parts[1])
	}
	parts = rcRegexp.FindStringSubmatch(suffix)
	if len(parts) > 1 {
		v.RC, _ = strconv.Atoi(parts[1])
	}
	fmt.Printf("version=%q v=%+v\n", version, v)
	return
}

func parseVersion(version string) ([]string, string, error) {
	ver := strings.TrimSpace(version)
	// strip off leading non-digits, such as 'v' or 'go'.
	p := prefixRegexp.FindStringSubmatch(ver)
	if len(p) > 2 {
		ver = p[2]
	}
	// populate parts array with digits in version, so
	// '1.2.3' becomes [1, 2, 3]
	var parts []string
	temp := ver
	for temp != "" {
		match := digitsRegexp.FindStringSubmatch(temp)
		if len(match) < 3 {
			break
		}
		parts = append(parts, match[1])
		temp = match[2]
		match = dotRegexp.FindStringSubmatch(temp)
		if len(match) < 2 {
			break
		}
		temp = match[1]
	}
	if len(parts) == 0 {
		return parts, "", fmt.Errorf("can not parse: %q", version)
	}
	// semver version always have 3 digit elements.
	for len(parts) < 3 {
		parts = append(parts, "0")
	}
	suffix := temp
	// Move any digit elements after the third element to the suffix.
	if len(parts) > 3 {
		rest := parts[3:]
		parts = parts[:3]
		if len(rest) > 0 {
			rests := strings.Join(rest, ".")
			if suffix[0] == '-' {
				suffix = suffix[1:]
			}

			suffix = "-" + rests + suffix
		}
	}
	// semver needs a dash prefix for the suffix
	if len(suffix) > 0 {
		if suffix[0] != '-' {
			suffix = "-" + suffix
		}
	}
	return parts, suffix, nil
}

// Semverize converts a version string to be semver compatible.
func Semverize(version string) (string, error) {
	parts, suffix, err := parseVersion(version)
	if err != nil {
		return "", err
	}
	return "v" + strings.Join(parts, ".") + suffix, nil
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

		if v1.Beta == 0 {
			v1.Beta = math.MaxInt
		}
		if v2.Beta == 0 {
			v2.Beta = math.MaxInt
		}
		if v1.Beta != v2.Beta {
			return v1.Beta > v2.Beta
		}
		if v1.RC == 0 {
			v1.RC = math.MaxInt
		}
		if v2.RC == 0 {
			v2.RC = math.MaxInt
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

		if v1.Beta != v2.Beta {
			return v1.Beta < v2.Beta
		}
		return v1.RC < v2.RC
	})
}
