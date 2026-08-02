package sotn

import (
	"testing"

	"github.com/stretchr/testify/require"
)

const testBaseCatalog = `
locale: en-US
strings:
  STR_ENEMY_ZOMBIE: Zombie
  STR_ITEM_HEART_MAX_UP: Heart max up
`

func TestGenerateStringCatalogHeader(t *testing.T) {
	catalog, err := ParseStringCatalog([]byte(testBaseCatalog))
	require.NoError(t, err)
	header, err := GenerateStringCatalogHeader(
		catalog, PlatformPSX, "SOTN_TEST_STRINGS_H")
	require.NoError(t, err)
	require.Equal(t, `/* Generated string catalog: en-US (psx). DO NOT EDIT. */
#ifndef SOTN_TEST_STRINGS_H
#define SOTN_TEST_STRINGS_H

/* Zombie */
#define STR_ENEMY_ZOMBIE "\x3A\x4F\x4D\x42\x49\x45\xFF"
/* Heart max up */
#define STR_ITEM_HEART_MAX_UP "\x28\x45\x41\x52\x54\x00\x4D\x41\x58\x00\x55\x50\xFF"

#endif /* SOTN_TEST_STRINGS_H */
`, string(header))
}

func TestStringCatalogOverlay(t *testing.T) {
	base, err := ParseStringCatalog([]byte(testBaseCatalog))
	require.NoError(t, err)
	overlay, err := ParseStringCatalog([]byte(`
locale: fr-FR
strings:
  STR_ENEMY_ZOMBIE: Mort-vivant
`))
	require.NoError(t, err)
	merged, err := ApplyStringCatalogOverlay(base, overlay)
	require.NoError(t, err)
	require.Equal(t, "fr-FR", merged.Locale)
	require.Equal(t, "Mort-vivant", merged.Strings["STR_ENEMY_ZOMBIE"])
	require.Equal(t, "Heart max up", merged.Strings["STR_ITEM_HEART_MAX_UP"])
}

func TestStringCatalogRejectsInvalidInput(t *testing.T) {
	_, err := ParseStringCatalog([]byte("locale: en-US\nstrings:\n  Zombie: Zombie\n"))
	require.ErrorContains(t, err, "invalid string ID")

	base, err := ParseStringCatalog([]byte(testBaseCatalog))
	require.NoError(t, err)
	overlay, err := ParseStringCatalog([]byte(
		"locale: modded\nstrings:\n  STR_NEW_ID: New string\n"))
	require.NoError(t, err)
	_, err = ApplyStringCatalogOverlay(base, overlay)
	require.ErrorContains(t, err, "unknown string ID")
}

func TestStringCatalogReportsEncodingFailure(t *testing.T) {
	catalog, err := ParseStringCatalog([]byte(
		"locale: test\nstrings:\n  STR_UNSUPPORTED: \u20ac\n"))
	require.NoError(t, err)
	_, err = GenerateStringCatalogHeader(catalog, PlatformPSX, "TEST_H")
	require.ErrorContains(t, err, "STR_UNSUPPORTED")
}

func TestStringCatalogSanitizesCommentsAndValidatesDirectCalls(t *testing.T) {
	catalog := StringCatalog{
		Locale:  "test */\n#define BAD 1\n/*",
		Strings: map[string]string{"STR_SAFE": "safe"},
	}
	header, err := GenerateStringCatalogHeader(catalog, PlatformPSX, "TEST_H")
	require.NoError(t, err)
	require.NotContains(t, string(header), "\n#define BAD")
	require.Contains(t, string(header), "test * / #define BAD 1 /*")

	_, err = GenerateStringCatalogHeader(
		StringCatalog{Locale: "test", Strings: map[string]string{"BAD": "value"}},
		PlatformPSX,
		"TEST_H",
	)
	require.ErrorContains(t, err, "invalid string ID")
}
