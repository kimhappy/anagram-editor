use std::{collections::BTreeMap, fmt};

use super::hid::version::Version;

const BLOCK: usize = 512;
const PLATFORM: &str = "pablito";
const VERSIONS: &str = "VERSIONS";
const SHA1SUMS: &str = "SHA1SUMS";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirmwarePackage {
    pub version: Option<Version>,
    pub members: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageError {
    NotTar,
    Truncated,
    MissingMember(String),
    WrongPlatform(String),
}

impl fmt::Display for PackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotTar => formatter.write_str("this is not an Anagram firmware package (.tar)"),
            Self::Truncated => formatter.write_str("the package is cut short; download it again"),
            Self::MissingMember(name) => write!(formatter, "the package has no {name}"),
            Self::WrongPlatform(platform) => {
                write!(
                    formatter,
                    "the package is for {platform}, not for the Anagram"
                )
            }
        }
    }
}

impl std::error::Error for PackageError {}

struct Member<'bytes> {
    name: String,
    content: &'bytes [u8],
}

pub fn inspect(bytes: &[u8]) -> Result<FirmwarePackage, PackageError> {
    let members = read_members(bytes)?;
    let text_of = |name: &str| {
        members
            .get(name)
            .map(|content| String::from_utf8_lossy(content).into_owned())
            .ok_or_else(|| PackageError::MissingMember(name.to_owned()))
    };
    let versions = key_values(&text_of(VERSIONS)?);
    if let Some(platform) = versions
        .get("platform")
        .filter(|platform| *platform != PLATFORM)
    {
        return Err(PackageError::WrongPlatform(platform.clone()));
    }
    let listed = text_of(SHA1SUMS)?;
    if let Some(missing) = listed
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .map(|(_, name)| {
            let name = name.trim_start().trim_start_matches('*');
            name.strip_prefix("./").unwrap_or(name)
        })
        .find(|name| !members.contains_key(*name))
    {
        return Err(PackageError::MissingMember(missing.to_owned()));
    }
    let version = ["version.build", "version"]
        .iter()
        .find_map(|key| versions.get(*key)?.parse::<Version>().ok());
    Ok(FirmwarePackage {
        version,
        members: members.into_keys().collect(),
    })
}

fn read_members(bytes: &[u8]) -> Result<BTreeMap<String, &[u8]>, PackageError> {
    let mut members = BTreeMap::new();
    let mut offset = 0;
    while let Some(header) = bytes.get(offset..offset + BLOCK) {
        if header.iter().all(|byte| *byte == 0) {
            return Ok(members);
        }
        let member = read_member(bytes, header, offset)?;
        let padded = member.content.len().div_ceil(BLOCK) * BLOCK;
        offset += BLOCK + padded;
        if !member.name.is_empty() {
            members.insert(member.name, member.content);
        }
    }
    if members.is_empty() {
        Err(PackageError::NotTar)
    } else {
        Err(PackageError::Truncated)
    }
}

fn read_member<'bytes>(
    bytes: &'bytes [u8],
    header: &[u8],
    offset: usize,
) -> Result<Member<'bytes>, PackageError> {
    if !has_valid_checksum(header) {
        return Err(PackageError::NotTar);
    }
    let size = octal(field(header, 124, 12)).ok_or(PackageError::NotTar)?;
    let start = offset + BLOCK;
    let end = start.checked_add(size).ok_or(PackageError::Truncated)?;
    let content = bytes.get(start..end).ok_or(PackageError::Truncated)?;
    let is_file = matches!(field(header, 156, 1), [b'0' | 0]);
    let name = [text(field(header, 345, 155)), text(field(header, 0, 100))]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    Ok(Member {
        name: if is_file {
            name.trim_start_matches("./").to_owned()
        } else {
            String::new()
        },
        content,
    })
}

fn field(header: &[u8], start: usize, length: usize) -> &[u8] {
    header.get(start..start + length).unwrap_or_default()
}

fn text(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw.split(|byte| *byte == 0).next().unwrap_or_default())
        .trim()
        .to_owned()
}

fn octal(raw: &[u8]) -> Option<usize> {
    let digits = text(raw);
    usize::from_str_radix(digits.trim(), 8).ok()
}

fn has_valid_checksum(header: &[u8]) -> bool {
    let stored = octal(field(header, 148, 8));
    let computed: usize = header
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            if (148..156).contains(&index) {
                usize::from(b' ')
            } else {
                usize::from(*byte)
            }
        })
        .sum();
    stored == Some(computed)
}

fn key_values(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{BLOCK, PackageError, inspect};

    fn header(name: &str, size: usize) -> Vec<u8> {
        let mut block = vec![0_u8; BLOCK];
        block[..name.len()].copy_from_slice(name.as_bytes());
        let size_field = format!("{size:011o}\0");
        block[124..136].copy_from_slice(size_field.as_bytes());
        block[156] = b'0';
        block[257..263].copy_from_slice(b"ustar\0");
        block[148..156].copy_from_slice(b"        ");
        let sum: usize = block.iter().map(|byte| usize::from(*byte)).sum();
        block[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
        block
    }

    fn tar(members: &[(&str, &[u8])]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (name, content) in members {
            bytes.extend(header(name, content.len()));
            bytes.extend_from_slice(content);
            bytes.resize(bytes.len().div_ceil(BLOCK) * BLOCK, 0);
        }
        bytes.extend([0; BLOCK * 2]);
        bytes
    }

    const SUMS: &[u8] = b"0123abcd  system.tar.xz.gpg\n";

    #[test]
    fn a_complete_package_reports_its_version() {
        let bytes = tar(&[
            ("VERSIONS", b"version.build=1.19.0.3\nplatform=pablito\n"),
            ("SHA1SUMS", SUMS),
            ("system.tar.xz.gpg", b"sealed"),
        ]);
        let package = inspect(&bytes).expect("inspects");
        assert_eq!(
            package.version.map(|version| version.to_string()),
            Some("1.19.0.3".to_owned())
        );
        assert_eq!(package.members.len(), 3);
    }

    #[test]
    fn packages_for_other_units_or_with_gaps_are_refused() {
        let other = tar(&[("VERSIONS", b"platform=other\n"), ("SHA1SUMS", SUMS)]);
        assert_eq!(
            inspect(&other),
            Err(PackageError::WrongPlatform("other".to_owned()))
        );
        let missing = tar(&[("VERSIONS", b"platform=pablito\n"), ("SHA1SUMS", SUMS)]);
        assert_eq!(
            inspect(&missing),
            Err(PackageError::MissingMember("system.tar.xz.gpg".to_owned()))
        );
        let whole = tar(&[
            ("VERSIONS", b"platform=pablito\n"),
            ("SHA1SUMS", SUMS),
            ("system.tar.xz.gpg", &[7; 1000]),
        ]);
        assert_eq!(inspect(&whole[..BLOCK * 4]), Err(PackageError::Truncated));
        assert_eq!(inspect(b"not a tar at all"), Err(PackageError::NotTar));
        let mut corrupt = whole;
        corrupt[10] = b'x';
        assert_eq!(inspect(&corrupt), Err(PackageError::NotTar));
    }
}
