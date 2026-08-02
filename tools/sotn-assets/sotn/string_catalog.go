package sotn

import (
	"fmt"
	"sort"
	"strings"

	"github.com/goccy/go-yaml"
)

// StringCatalog separates stable semantic string identities from localized
// text. Encoding remains a property of the selected target platform.
type StringCatalog struct {
	Locale  string            `yaml:"locale"`
	Strings map[string]string `yaml:"strings"`
}

// ParseStringCatalog reads and validates one declarative catalog.
func ParseStringCatalog(data []byte) (StringCatalog, error) {
	var catalog StringCatalog
	if err := yaml.Unmarshal(data, &catalog); err != nil {
		return StringCatalog{}, fmt.Errorf("parse string catalog: %w", err)
	}
	if err := validateStringCatalog(catalog); err != nil {
		return StringCatalog{}, err
	}
	return catalog, nil
}

func validateStringCatalog(catalog StringCatalog) error {
	if strings.TrimSpace(catalog.Locale) == "" {
		return fmt.Errorf("string catalog locale must not be empty")
	}
	if len(catalog.Strings) == 0 {
		return fmt.Errorf("string catalog must contain at least one string")
	}
	for key := range catalog.Strings {
		if !validStringID(key) {
			return fmt.Errorf(
				"invalid string ID %q; expected STR_[A-Z0-9_]+", key)
		}
	}
	return nil
}

// ApplyStringCatalogOverlay replaces text by stable ID. Overlays may be
// partial, but cannot invent identities that are absent from the base catalog.
func ApplyStringCatalogOverlay(base, overlay StringCatalog) (StringCatalog, error) {
	merged := StringCatalog{
		Locale:  overlay.Locale,
		Strings: make(map[string]string, len(base.Strings)),
	}
	for key, value := range base.Strings {
		merged.Strings[key] = value
	}
	for key, value := range overlay.Strings {
		if _, exists := base.Strings[key]; !exists {
			return StringCatalog{}, fmt.Errorf(
				"overlay %s defines unknown string ID %s", overlay.Locale, key)
		}
		merged.Strings[key] = value
	}
	return merged, nil
}

// GenerateStringCatalogHeader emits a target-specific, translation-unit-safe
// header. Readable text remains beside every byte-encoded macro.
func GenerateStringCatalogHeader(
	catalog StringCatalog,
	platform Platform,
	guard string,
) ([]byte, error) {
	if err := validateStringCatalog(catalog); err != nil {
		return nil, err
	}
	if !validHeaderGuard(guard) {
		return nil, fmt.Errorf("invalid C header guard %q", guard)
	}
	keys := make([]string, 0, len(catalog.Strings))
	for key := range catalog.Strings {
		keys = append(keys, key)
	}
	sort.Strings(keys)

	var output strings.Builder
	fmt.Fprintf(&output, "/* Generated string catalog: %s (%s). DO NOT EDIT. */\n", cComment(catalog.Locale), platform)
	fmt.Fprintf(&output, "#ifndef %s\n#define %s\n\n", guard, guard)
	for _, key := range keys {
		text := catalog.Strings[key]
		encoded, err := EncodeString(text, platform)
		if err != nil {
			return nil, fmt.Errorf("encode %s for %s: %w", key, platform, err)
		}
		fmt.Fprintf(&output, "/* %s */\n#define %s \"", cComment(text), key)
		for _, value := range encoded {
			fmt.Fprintf(&output, "\\x%02X", value)
		}
		output.WriteString("\"\n")
	}
	fmt.Fprintf(&output, "\n#endif /* %s */\n", guard)
	return []byte(output.String()), nil
}

func validStringID(value string) bool {
	if !strings.HasPrefix(value, "STR_") || len(value) == len("STR_") {
		return false
	}
	for _, character := range value[len("STR_"):] {
		if character != '_' && (character < '0' || character > '9') &&
			(character < 'A' || character > 'Z') {
			return false
		}
	}
	return true
}

func validHeaderGuard(value string) bool {
	if value == "" || (value[0] < 'A' || value[0] > 'Z') && value[0] != '_' {
		return false
	}
	for _, character := range value {
		if character != '_' && (character < '0' || character > '9') &&
			(character < 'A' || character > 'Z') {
			return false
		}
	}
	return true
}

func cComment(value string) string {
	value = strings.NewReplacer("\r", " ", "\n", " ", "*/", "* /").Replace(value)
	return strings.TrimSpace(value)
}
