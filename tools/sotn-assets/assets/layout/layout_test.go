package layout

import (
	"bytes"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/xeeynamo/sotn-decomp/tools/sotn-assets/assets"
	"github.com/xeeynamo/sotn-decomp/tools/sotn-assets/datarange"
	"github.com/xeeynamo/sotn-decomp/tools/sotn-assets/psx"
)

func TestExtractPropagatesLayoutReadError(t *testing.T) {
	base := psx.Addr(0x80180000)
	data := make([]byte, 0x40)
	binary.LittleEndian.PutUint32(data[0x1C:], uint32(base.Sum(0x40)))

	err := Handler.Extract(assets.ExtractArgs{
		Data:     data,
		AssetDir: t.TempDir(),
		Name:     "entity_layouts",
		RamBase:  base,
	})
	assert.Error(t, err)
}

func putLayoutMarker(data []byte, off int) {
	binary.LittleEndian.PutUint16(data[off:], 0xFFFE)
	binary.LittleEndian.PutUint16(data[off+2:], 0xFFFE)
}

func putLayoutBlock(data []byte, off int) {
	putLayoutMarker(data, off)
	binary.LittleEndian.PutUint16(data[off+10:], 0xFFFF)
	binary.LittleEndian.PutUint16(data[off+12:], 0xFFFF)
}

func buildLayoutsForTest(t *testing.T, el layouts) (string, error) {
	t.Helper()
	assetDir := t.TempDir()
	srcDir := t.TempDir()
	serialized, err := json.Marshal(el)
	require.NoError(t, err)
	require.NoError(t, os.WriteFile(
		filepath.Join(assetDir, "entity_layouts.json"), serialized, 0o644))
	return srcDir, Handler.Build(assets.BuildArgs{
		AssetDir: assetDir,
		SrcDir:   srcDir,
		Name:     "entity_layouts",
		OvlName:  "dummy",
	})
}

func TestReadEntityLayoutPreservesUnclaimedRanges(t *testing.T) {
	const count = 2
	base := psx.Addr(0x80180000)
	data := make([]byte, 0xA0)

	blockOffsets := []int{0x20, 0x40, 0x60, 0x80}
	for i, off := range blockOffsets {
		binary.LittleEndian.PutUint32(data[i*4:], uint32(base.Sum(off)))
		putLayoutBlock(data, off)
	}
	// Pointers to room-referenced blocks do not claim the gap.
	for _, gap := range [][2]int{{0x34, 0x20}, {0x74, 0x60}} {
		for off := gap[0]; off < gap[0]+0xC; off += 4 {
			binary.LittleEndian.PutUint32(data[off:], uint32(base.Sum(gap[1])))
		}
	}

	layouts, ranges, err := readEntityLayout(
		bytes.NewReader(data), "dummy", base, base, count, true)
	require.NoError(t, err)
	assert.Len(t, layouts.Entities, count)
	assert.Empty(t, layouts.UnusedPointerTables)
	assert.Equal(t, []datarange.DataRange{
		datarange.New(base, base.Sum(0x10)),
		datarange.New(base.Sum(0x20), base.Sum(0x34)),
		datarange.New(base.Sum(0x40), base.Sum(0x54)),
		datarange.New(base.Sum(0x60), base.Sum(0x74)),
		datarange.New(base.Sum(0x80), base.Sum(0x94)),
	}, ranges)
}

func TestReadEntityLayoutClaimsUnusedBlocksAndPointers(t *testing.T) {
	const count = 2
	base := psx.Addr(0x80180000)
	data := make([]byte, 0xA0)

	// Each axis has one unreferenced block and a pointer to it.
	for axis, dataBegin := range []int{0x20, 0x60} {
		binary.LittleEndian.PutUint32(data[axis*8:], uint32(base.Sum(dataBegin)))
		binary.LittleEndian.PutUint32(data[axis*8+4:], uint32(base.Sum(dataBegin+0x2C)))
		putLayoutBlock(data, dataBegin)
		putLayoutBlock(data, dataBegin+0x14)
		binary.LittleEndian.PutUint32(
			data[dataBegin+0x28:], uint32(base.Sum(dataBegin+0x14)))
		putLayoutBlock(data, dataBegin+0x2C)
	}

	layouts, ranges, err := readEntityLayout(
		bytes.NewReader(data), "dummy", base, base, count, true)
	require.NoError(t, err)
	assert.Len(t, layouts.Entities, 3)
	assert.Equal(t, []int{0, 2}, layouts.Indices)
	assert.Equal(t, []unusedPointerTable{{AfterBlock: 1, Targets: []int{1}}},
		layouts.UnusedPointerTables)

	consolidated, err := datarange.ConsolidateDataRanges(ranges[1:])
	require.NoError(t, err)
	assert.Equal(t, []datarange.DataRange{
		datarange.New(base.Sum(0x20), base.Sum(0xA0)),
	}, consolidated)
}

func TestReadEntityLayoutClaimsSingleTruncatedMarker(t *testing.T) {
	const count = 2
	base := psx.Addr(0x80180000)
	data := make([]byte, 0xB0)

	for axis, dataBegin := range []int{0x20, 0x70} {
		binary.LittleEndian.PutUint32(data[axis*8:], uint32(base.Sum(dataBegin)))
		binary.LittleEndian.PutUint32(
			data[axis*8+4:], uint32(base.Sum(dataBegin+0x1E)))
		putLayoutBlock(data, dataBegin)
		putLayoutMarker(data, dataBegin+0x14)
		putLayoutBlock(data, dataBegin+0x1E)
	}

	layouts, ranges, err := readEntityLayout(
		bytes.NewReader(data), "dummy", base, base, count, true)
	require.NoError(t, err)
	assert.Len(t, layouts.Entities, 3)
	assert.Equal(t, []int{0, 2}, layouts.Indices)
	assert.Len(t, layouts.Entities[1], 1)
	assert.Empty(t, layouts.UnusedPointerTables)
	consolidated, err := datarange.ConsolidateDataRanges(ranges[1:])
	require.NoError(t, err)
	assert.Equal(t, []datarange.DataRange{
		datarange.New(base.Sum(0x20), base.Sum(0x54)),
		datarange.New(base.Sum(0x70), base.Sum(0xA4)),
	}, consolidated)
}

func TestReadEntityLayoutRollsBackRejectedTruncatedBlock(t *testing.T) {
	const count = 2
	base := psx.Addr(0x80180000)
	data := make([]byte, 0x110)

	for axis, dataBegin := range []int{0x20, 0xA0} {
		binary.LittleEndian.PutUint32(data[axis*8:], uint32(base.Sum(dataBegin)))
		binary.LittleEndian.PutUint32(
			data[axis*8+4:], uint32(base.Sum(dataBegin+0x46)))
		putLayoutBlock(data, dataBegin)
		putLayoutBlock(data, dataBegin+0x14)
		// Two entries without a terminator must be rejected.
		putLayoutMarker(data, dataBegin+0x28)
		putLayoutBlock(data, dataBegin+0x46)
	}

	layouts, ranges, err := readEntityLayout(
		bytes.NewReader(data), "dummy", base, base, count, true)
	require.NoError(t, err)
	assert.Len(t, layouts.Entities, 2)
	assert.Equal(t, []int{0, 1}, layouts.Indices)
	assert.Empty(t, layouts.UnusedPointerTables)
	assert.Equal(t, []datarange.DataRange{
		datarange.New(base, base.Sum(0x10)),
		datarange.New(base.Sum(0x20), base.Sum(0x34)),
		datarange.New(base.Sum(0x66), base.Sum(0x7C)),
		datarange.New(base.Sum(0xA0), base.Sum(0xB4)),
		datarange.New(base.Sum(0xE6), base.Sum(0xFC)),
	}, ranges)
}

func TestBuildSplitsTheArrayAtAnUnusedPointerTable(t *testing.T) {
	block := []layoutEntry{{X: -2, Y: -2, ID: "0x00"}, {X: -1, Y: -1, ID: "0x00"}}
	srcDir, err := buildLayoutsForTest(t, layouts{
		Entities:            [][]layoutEntry{block, block, block},
		Indices:             []int{0, 2},
		UnusedPointerTables: []unusedPointerTable{{AfterBlock: 1, Targets: []int{1}}},
	})
	require.NoError(t, err)

	laydef, err := os.ReadFile(filepath.Join(srcDir, "gen", "e_laydef.c"))
	require.NoError(t, err)
	// The symbol name includes the output-directory hash.
	_, rest, ok := strings.Cut(string(laydef), "extern LayoutEntity ")
	require.True(t, ok)
	sym, _, ok := strings.Cut(rest, "_x[];")
	require.True(t, ok)

	assert.Contains(t, string(laydef), fmt.Sprintf(`extern LayoutEntity %[1]s_x[];
extern LayoutEntity %[1]s_x_2[];
LayoutEntity* entityLayoutHorizontal[] = {
    &%[1]s_x[0],
    &%[1]s_x_2[0],
};
`, sym))

	entry := "    0xFFFE, 0xFFFE, 0x00 | 0x0000, 0x0000, 0x0000,\n" +
		"    0xFFFF, 0xFFFF, 0x00 | 0x0000, 0x0000, 0x0000,\n"
	generated, err := os.ReadFile(filepath.Join(srcDir, "gen", "e_layout.c"))
	require.NoError(t, err)
	// Pointer offsets are scaled to the generated u16 array.
	assert.Contains(t, string(generated), fmt.Sprintf(`u16 %[1]s_x[] = {
// Offset 0, Room 0x00
%[2]s// Offset 2, No Room Found
%[2]s};
LayoutEntity* %[1]s_x_unused_ptrs[] = {
    (LayoutEntity*)&%[1]s_x[10],
};
u16 %[1]s_x_2[] = {
// Offset 0, Room 0x01
%[2]s};
`, sym, entry))
}

func TestBuildOnlyAllowsSingleUnreferencedMarker(t *testing.T) {
	marker := layoutEntry{X: -2, Y: -2, ID: "0x00"}
	entity := layoutEntry{X: 1, Y: 2, ID: "0x00"}
	terminator := layoutEntry{X: -1, Y: -1, ID: "0x00"}
	terminated := []layoutEntry{marker, terminator}

	t.Run("unreferenced marker", func(t *testing.T) {
		_, err := buildLayoutsForTest(t, layouts{
			Entities: [][]layoutEntry{terminated, []layoutEntry{marker}, terminated},
			Indices:  []int{0, 2},
		})
		require.NoError(t, err)
	})

	t.Run("referenced marker", func(t *testing.T) {
		_, err := buildLayoutsForTest(t, layouts{
			Entities: [][]layoutEntry{{marker}},
			Indices:  []int{0},
		})
		assert.ErrorContains(t, err, "needs to have a X:-1 and Y:-1 entry")
	})

	t.Run("long unreferenced block", func(t *testing.T) {
		_, err := buildLayoutsForTest(t, layouts{
			Entities: [][]layoutEntry{
				terminated,
				[]layoutEntry{marker, entity},
				terminated,
			},
			Indices: []int{0, 2},
		})
		assert.ErrorContains(t, err, "may only omit its terminator")
	})
}

func TestPlanLayoutHandlesSeveralSplits(t *testing.T) {
	plan, err := planLayout(layouts{
		Entities: [][]layoutEntry{
			make([]layoutEntry, 2), make([]layoutEntry, 2),
			make([]layoutEntry, 2), make([]layoutEntry, 2),
		},
		UnusedPointerTables: []unusedPointerTable{
			{AfterBlock: 1, Targets: []int{1}},
			{AfterBlock: 2, Targets: []int{1}},
		},
	})
	require.NoError(t, err)
	assert.Equal(t, []int{0, 0, 1, 2}, plan.arrayOf)
	assert.Equal(t, []int{0, 2, 0, 0}, plan.entryOf)
	assert.Equal(t, "sym_x", arrayName("sym", "x", 0))
	assert.Equal(t, "sym_x_3", arrayName("sym", "x", 2))
}

func TestPlanLayoutRejectsAMisalignedSplit(t *testing.T) {
	_, err := planLayout(layouts{
		Entities:            [][]layoutEntry{make([]layoutEntry, 1), make([]layoutEntry, 1)},
		UnusedPointerTables: []unusedPointerTable{{AfterBlock: 0, Targets: []int{0}}},
	})
	assert.ErrorContains(t, err, "do not fill a multiple of 4 bytes")
}

func TestPlanLayoutRejectsInvalidUnusedPointerTables(t *testing.T) {
	tests := []struct {
		name  string
		table unusedPointerTable
	}{
		{name: "no targets", table: unusedPointerTable{AfterBlock: 0}},
		{name: "negative target", table: unusedPointerTable{AfterBlock: 0, Targets: []int{-1}}},
		{name: "forward target", table: unusedPointerTable{AfterBlock: 0, Targets: []int{1}}},
		{name: "out of range target", table: unusedPointerTable{AfterBlock: 1, Targets: []int{3}}},
		{name: "table after final block", table: unusedPointerTable{AfterBlock: 2, Targets: []int{0}}},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			_, err := planLayout(layouts{
				Entities:            make([][]layoutEntry, 3),
				UnusedPointerTables: []unusedPointerTable{test.table},
			})
			assert.Error(t, err)
		})
	}
}
