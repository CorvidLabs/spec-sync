//! Fail-closed issue-reference validation over complete real YAML.

use super::frontmatter::split_frontmatter;
use serde::Deserialize;
use serde::de::{IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::fmt;

const INVALID_YAML_FRONTMATTER_ERROR: &str = "invalid YAML frontmatter";
const DUPLICATE_YAML_KEY_ERROR: &str = "duplicate YAML frontmatter key";
const DUPLICATE_IMPLEMENTS_ERROR: &str = "duplicate `implements` issue-reference field";
const DUPLICATE_TRACKS_ERROR: &str = "duplicate `tracks` issue-reference field";
const INVALID_IMPLEMENTS_SHAPE_ERROR: &str =
    "`implements` must be a list of unsigned issue numbers";
const INVALID_TRACKS_SHAPE_ERROR: &str = "`tracks` must be a list of unsigned issue numbers";
const INVALID_IMPLEMENTS_NUMBER_ERROR: &str =
    "`implements` contains an invalid unsigned issue number";
const INVALID_TRACKS_NUMBER_ERROR: &str = "`tracks` contains an invalid unsigned issue number";

#[derive(Default)]
struct CheckedIssueReferences {
    implements: Vec<u64>,
    tracks: Vec<u64>,
}

struct ImplementsIssueNumbers(Vec<u64>);
struct TracksIssueNumbers(Vec<u64>);

impl<'de> Deserialize<'de> for CheckedIssueReferences {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CheckedIssueReferencesVisitor)
    }
}

struct CheckedIssueReferencesVisitor;

impl<'de> Visitor<'de> for CheckedIssueReferencesVisitor {
    type Value = CheckedIssueReferences;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("YAML frontmatter mapping")
    }

    fn visit_map<Mapping>(self, mut mapping: Mapping) -> Result<Self::Value, Mapping::Error>
    where
        Mapping: MapAccess<'de>,
    {
        let mut references = CheckedIssueReferences::default();
        let mut implements_seen = false;
        let mut tracks_seen = false;

        while let Some(key) = mapping.next_key::<String>()? {
            match key.as_str() {
                "implements" => {
                    if implements_seen {
                        return Err(serde::de::Error::custom(DUPLICATE_IMPLEMENTS_ERROR));
                    }
                    implements_seen = true;
                    references.implements = mapping.next_value::<ImplementsIssueNumbers>()?.0;
                }
                "tracks" => {
                    if tracks_seen {
                        return Err(serde::de::Error::custom(DUPLICATE_TRACKS_ERROR));
                    }
                    tracks_seen = true;
                    references.tracks = mapping.next_value::<TracksIssueNumbers>()?.0;
                }
                _ => {
                    mapping.next_value::<IgnoredAny>()?;
                }
            }
        }

        Ok(references)
    }
}

impl<'de> Deserialize<'de> for ImplementsIssueNumbers {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserialize_issue_number_list(
            deserializer,
            INVALID_IMPLEMENTS_SHAPE_ERROR,
            INVALID_IMPLEMENTS_NUMBER_ERROR,
        )
        .map(Self)
    }
}

impl<'de> Deserialize<'de> for TracksIssueNumbers {
    fn deserialize<Deserializer>(deserializer: Deserializer) -> Result<Self, Deserializer::Error>
    where
        Deserializer: serde::Deserializer<'de>,
    {
        deserialize_issue_number_list(
            deserializer,
            INVALID_TRACKS_SHAPE_ERROR,
            INVALID_TRACKS_NUMBER_ERROR,
        )
        .map(Self)
    }
}

fn deserialize_issue_number_list<'de, Deserializer>(
    deserializer: Deserializer,
    shape_error: &'static str,
    number_error: &'static str,
) -> Result<Vec<u64>, Deserializer::Error>
where
    Deserializer: serde::Deserializer<'de>,
{
    deserializer.deserialize_any(IssueNumberListVisitor {
        shape_error,
        number_error,
    })
}

struct IssueNumberListVisitor {
    shape_error: &'static str,
    number_error: &'static str,
}

impl<'de> Visitor<'de> for IssueNumberListVisitor {
    type Value = Vec<u64>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("list of positive unsigned issue numbers")
    }

    fn visit_bool<Error>(self, _value: bool) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_i64<Error>(self, _value: i64) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_u64<Error>(self, _value: u64) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_f64<Error>(self, _value: f64) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_str<Error>(self, _value: &str) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_string<Error>(self, _value: String) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_none<Error>(self) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_unit<Error>(self) -> Result<Self::Value, Error>
    where
        Error: serde::de::Error,
    {
        Err(Error::custom(self.shape_error))
    }

    fn visit_map<Mapping>(self, _mapping: Mapping) -> Result<Self::Value, Mapping::Error>
    where
        Mapping: MapAccess<'de>,
    {
        Err(serde::de::Error::custom(self.shape_error))
    }

    fn visit_seq<Sequence>(self, mut sequence: Sequence) -> Result<Self::Value, Sequence::Error>
    where
        Sequence: SeqAccess<'de>,
    {
        let mut numbers = Vec::new();
        while let Some(number) = sequence
            .next_element::<u64>()
            .map_err(|_| serde::de::Error::custom(self.number_error))?
        {
            if number == 0 {
                return Err(serde::de::Error::custom(self.number_error));
            }
            numbers.push(number);
        }
        Ok(numbers)
    }
}

/// Parse and validate top-level GitHub issue references from YAML frontmatter.
///
/// Unknown extension fields remain valid YAML but are otherwise ignored. Errors are
/// intentionally stable and content-free so callers can safely expose them.
pub fn parse_checked_issue_references(content: &str) -> Result<(Vec<u64>, Vec<u64>), String> {
    let content = content.trim_start_matches('\u{feff}');
    // Reads the block through the one delimiter scan (#716). The hand-rolled pair of
    // `strip_prefix`/`split_once` chains this replaces required BOTH delimiters to carry the SAME
    // line ending, so a document opened with LF and closed with CRLF was reported as having no
    // frontmatter at all; and a closer padded with a trailing space was walked past, feeding body
    // prose to the YAML parser and reporting the references as invalid when they were not.
    //
    // An EMPTY block is refused exactly as it was before: `split_once` could not produce one, so
    // `---` immediately followed by `---` has always been "missing or malformed" here. That is
    // not an accident worth inheriting silently, so it is asserted rather than assumed.
    let Some(frontmatter) = split_frontmatter(content)
        .map(|(frontmatter, _)| frontmatter)
        .filter(|frontmatter| !frontmatter.is_empty())
    else {
        return Err("missing or malformed YAML frontmatter".to_string());
    };

    let options = serde_saphyr::options! {
        duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
        with_snippet: false,
    };
    // The scan returns the block with its final line ending; the previous split did not. Trim so
    // the YAML handed to the deserializer is byte-identical to what it received before.
    let frontmatter = format!("{}\n", frontmatter.trim_end_matches(['\r', '\n']));
    let references =
        serde_saphyr::from_str_with_options::<CheckedIssueReferences>(&frontmatter, options)
            .map_err(checked_issue_reference_error)?;

    Ok((references.implements, references.tracks))
}

fn checked_issue_reference_error(error: serde_saphyr::Error) -> String {
    match error.without_snippet() {
        serde_saphyr::Error::Message { msg, .. }
            if [
                DUPLICATE_IMPLEMENTS_ERROR,
                DUPLICATE_TRACKS_ERROR,
                INVALID_IMPLEMENTS_SHAPE_ERROR,
                INVALID_TRACKS_SHAPE_ERROR,
                INVALID_IMPLEMENTS_NUMBER_ERROR,
                INVALID_TRACKS_NUMBER_ERROR,
            ]
            .contains(&msg.as_str()) =>
        {
            msg.clone()
        }
        serde_saphyr::Error::DuplicateMappingKey { key, .. } => match key.as_deref() {
            Some("implements") => DUPLICATE_IMPLEMENTS_ERROR.to_string(),
            Some("tracks") => DUPLICATE_TRACKS_ERROR.to_string(),
            _ => DUPLICATE_YAML_KEY_ERROR.to_string(),
        },
        _ => INVALID_YAML_FRONTMATTER_ERROR.to_string(),
    }
}

#[cfg(test)]
#[path = "tests/checked_issues.rs"]
mod tests;
