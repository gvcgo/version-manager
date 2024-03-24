package utils

import (
	"errors"
	"sort"
	"strconv"
	"strings"
)

// Version represents a version number.
type Version struct {
	Major, Minor, Patch int
}

// ParseVersion parses a version string into a Version struct.
func ParseVersion(version string) (Version, error) {
	parts := strings.Split(version, ".")
	if len(parts) != 3 {
		return Version{}, errors.New("invalid version format")
	}
	major, err := strconv.Atoi(parts[0])
	if err != nil {
		return Version{}, err
	}
	minor, err := strconv.Atoi(parts[1])
	if err != nil {
		return Version{}, err
	}
	patch, err := strconv.Atoi(parts[2])
	if err != nil {
		return Version{}, err
	}
	return Version{Major: major, Minor: minor, Patch: patch}, nil
}

// SortVersions sorts a slice of version strings in descending order.
func SortVersions(versions []string) {
	sort.Slice(versions, func(i, j int) bool {
		v1, err := ParseVersion(versions[i])
		if err != nil {
			return false
		}
		v2, err := ParseVersion(versions[j])
		if err != nil {
			return false
		}
		if v1.Major != v2.Major {
			return v1.Major > v2.Major
		}
		if v1.Minor != v2.Minor {
			return v1.Minor > v2.Minor
		}
		return v1.Patch > v2.Patch
	})
}
