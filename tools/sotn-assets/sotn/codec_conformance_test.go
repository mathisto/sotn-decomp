package sotn

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
	"github.com/xeeynamo/sotn-decomp/tools/sotn-assets/psx"
)

type codecVectors struct {
	Encode        []codecEncodeVector        `json:"encode"`
	Decode        []codecDecodeVector        `json:"decode"`
	InvalidEncode []codecInvalidEncodeVector `json:"invalid_encode"`
	InvalidDecode []codecInvalidDecodeVector `json:"invalid_decode"`
}

type codecEncodeVector struct {
	Name string  `json:"name"`
	Text string  `json:"text"`
	PSX  *string `json:"psx"`
	PSP  *string `json:"psp"`
}

type codecDecodeVector struct {
	Name  string  `json:"name"`
	Bytes string  `json:"bytes"`
	PSX   *string `json:"psx"`
	PSP   *string `json:"psp"`
}

type codecInvalidEncodeVector struct {
	Name      string   `json:"name"`
	Text      string   `json:"text"`
	Platforms []string `json:"platforms"`
}

type codecInvalidDecodeVector struct {
	Name      string   `json:"name"`
	Bytes     string   `json:"bytes"`
	Platforms []string `json:"platforms"`
}

func loadCodecVectors(t *testing.T) codecVectors {
	t.Helper()
	data, err := os.ReadFile(filepath.Join("..", "..", "sotn_codec", "conformance.json"))
	require.NoError(t, err)
	var vectors codecVectors
	require.NoError(t, json.Unmarshal(data, &vectors))
	return vectors
}

func vectorPlatforms(psxValue, pspValue *string) map[Platform]*string {
	return map[Platform]*string{
		PlatformPSX: psxValue,
		PlatformPSP: pspValue,
	}
}

func TestSharedCodecEncodeVectors(t *testing.T) {
	for _, vector := range loadCodecVectors(t).Encode {
		for platform, expected := range vectorPlatforms(vector.PSX, vector.PSP) {
			if expected == nil {
				continue
			}
			t.Run(vector.Name+"/"+string(platform), func(t *testing.T) {
				want, err := hex.DecodeString(*expected)
				require.NoError(t, err)
				got, err := EncodeString(vector.Text, platform)
				require.NoError(t, err)
				require.Equal(t, want, got)
			})
		}
	}
}

func TestSharedCodecDecodeVectors(t *testing.T) {
	const base = psx.Addr(0x800A0000)
	for _, vector := range loadCodecVectors(t).Decode {
		data, err := hex.DecodeString(vector.Bytes)
		require.NoError(t, err)
		for platform, expected := range vectorPlatforms(vector.PSX, vector.PSP) {
			if expected == nil {
				continue
			}
			t.Run(vector.Name+"/"+string(platform), func(t *testing.T) {
				got, err := DecodeString(data, base, base, platform)
				require.NoError(t, err)
				require.Equal(t, *expected, got)
			})
		}
	}
}

func TestSharedCodecInvalidVectors(t *testing.T) {
	const base = psx.Addr(0x800A0000)
	vectors := loadCodecVectors(t)
	for _, vector := range vectors.InvalidEncode {
		for _, platform := range vector.Platforms {
			_, err := EncodeString(vector.Text, Platform(platform))
			require.Error(t, err, vector.Name+"/"+platform)
		}
	}
	for _, vector := range vectors.InvalidDecode {
		data, err := hex.DecodeString(vector.Bytes)
		require.NoError(t, err)
		for _, platform := range vector.Platforms {
			_, err := DecodeString(data, base, base, Platform(platform))
			require.Error(t, err, vector.Name+"/"+platform)
		}
	}
}
