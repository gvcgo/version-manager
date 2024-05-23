package table

import (
	"math"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/muesli/ansi"
)

func getLines(s string) (lines []string, widest int) {
	lines = strings.Split(s, "\n")

	for _, l := range lines {
		w := ansi.PrintableRuneWidth(l)
		if widest < w {
			widest = w
		}
	}
	for idx, line := range lines {
		if strings.Contains(line, "───") {
			lines[idx] = strings.Repeat(string('‑'), ansi.PrintableRuneWidth(line))
		}
	}
	return lines, widest
}

func JoinHorizontal(pos lipgloss.Position, strs ...string) string {
	if len(strs) == 0 {
		return ""
	}
	if len(strs) == 1 {
		return strs[0]
	}

	var (
		// Groups of strings broken into multiple lines
		blocks = make([][]string, len(strs))

		// Max line widths for the above text blocks
		maxWidths = make([]int, len(strs))

		// Height of the tallest block
		maxHeight int
	)

	// Break text blocks into lines and get max widths for each text block
	for i, str := range strs {
		blocks[i], maxWidths[i] = getLines(str)
		if len(blocks[i]) > maxHeight {
			maxHeight = len(blocks[i])
		}
	}

	// Add extra lines to make each side the same height
	for i := range blocks {
		if len(blocks[i]) >= maxHeight {
			continue
		}

		extraLines := make([]string, maxHeight-len(blocks[i]))

		switch pos { //nolint:exhaustive
		case lipgloss.Top:
			blocks[i] = append(blocks[i], extraLines...)

		case lipgloss.Bottom:
			blocks[i] = append(extraLines, blocks[i]...)

		default: // Somewhere in the middle
			n := len(extraLines)
			split := int(math.Round(float64(n) * float64(pos)))
			top := n - split
			bottom := n - top

			blocks[i] = append(extraLines[top:], blocks[i]...)
			blocks[i] = append(blocks[i], extraLines[bottom:]...)
		}
	}

	// Merge lines
	var b strings.Builder
	for i := range blocks[0] { // remember, all blocks have the same number of members now
		for j, block := range blocks {
			b.WriteString(block[i])

			// Also make lines the same length

			b.WriteString(strings.Repeat(" ", maxWidths[j]-ansi.PrintableRuneWidth(block[i])))
		}
		if i < len(blocks[0])-1 {
			b.WriteRune('\n')
		}
	}

	return b.String()
}

func JoinVertical(pos lipgloss.Position, strs ...string) string {
	if len(strs) == 0 {
		return ""
	}
	if len(strs) == 1 {
		return strs[0]
	}

	var (
		blocks   = make([][]string, len(strs))
		maxWidth int
	)

	for i := range strs {
		var w int
		blocks[i], w = getLines(strs[i])
		if w > maxWidth {
			maxWidth = w
		}
	}

	var b strings.Builder
	for i, block := range blocks {
		for j, line := range block {
			w := maxWidth - ansi.PrintableRuneWidth(line)

			switch pos { //nolint:exhaustive
			case lipgloss.Left:
				b.WriteString(line)
				b.WriteString(strings.Repeat(" ", w))

			case lipgloss.Right:
				b.WriteString(strings.Repeat(" ", w))
				b.WriteString(line)

			default: // Somewhere in the middle
				if w < 1 {
					b.WriteString(line)
					break
				}

				split := int(math.Round(float64(w) * float64(pos)))
				right := w - split
				left := w - right

				b.WriteString(strings.Repeat(" ", left))
				b.WriteString(line)
				b.WriteString(strings.Repeat(" ", right))
			}

			// Write a newline as long as we're not on the last line of the
			// last block.
			if !(i == len(blocks)-1 && j == len(block)-1) {
				b.WriteRune('\n')
			}
		}
	}

	return b.String()
}
