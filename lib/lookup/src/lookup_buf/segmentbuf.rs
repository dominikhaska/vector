use crate::{field, LookSegment, Segment};
use inherent::inherent;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct FieldBuf {
    pub name: String,
    // This is a very lazy optimization to avoid having to scan for escapes.
    pub requires_quoting: bool,
}

impl FieldBuf {
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl Display for FieldBuf {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        if self.requires_quoting {
            write!(formatter, r#""{}""#, self.name)
        } else {
            write!(formatter, "{}", self.name)
        }
    }
}

impl From<String> for FieldBuf {
    fn from(mut name: String) -> Self {
        let mut requires_quoting = false;

        if name.starts_with('\"') {
            // There is unfortunately no way to make an owned substring of a string.
            // So we have to take a slice and clone it.
            let len = name.len();
            name = name[1..len - 1].to_string();
            requires_quoting = true;
        } else if !field::is_valid_fieldname(&name) {
            requires_quoting = true
        }

        Self {
            name,
            requires_quoting,
        }
    }
}

impl From<&str> for FieldBuf {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}

/// `SegmentBuf`s are chunks of a `LookupBuf`.
///
/// They represent either a field or an index. A sequence of `SegmentBuf`s can become a `LookupBuf`.
///
/// This is the owned, allocated side of a `Segement` for `LookupBuf.` It owns its fields unlike `Lookup`. Think of `String` to `&str` or `PathBuf` to `Path`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum SegmentBuf {
    Field(FieldBuf),
    Index(isize), // Indexes can be negative.
    // Coalesces hold multiple segment sets.
    Coalesce(Vec<FieldBuf>),
}

impl SegmentBuf {
    pub fn as_segment(&self) -> Segment {
        match self {
            SegmentBuf::Field(field) => Segment::field(field.into()),
            SegmentBuf::Index(i) => Segment::index(*i),
            SegmentBuf::Coalesce(v) => {
                Segment::coalesce(v.iter().map(|field| field.into()).collect())
            }
        }
    }
}

#[inherent(pub)]
impl<'a> LookSegment<'a> for SegmentBuf {
    type Field = FieldBuf;

    fn field(field: FieldBuf) -> SegmentBuf {
        SegmentBuf::Field(field)
    }

    fn is_field(&self) -> bool {
        matches!(self, SegmentBuf::Field(_))
    }

    fn index(v: isize) -> SegmentBuf {
        SegmentBuf::Index(v)
    }

    fn is_index(&self) -> bool {
        matches!(self, SegmentBuf::Index(_))
    }

    fn coalesce(v: Vec<FieldBuf>) -> SegmentBuf {
        SegmentBuf::Coalesce(v)
    }

    fn is_coalesce(&self) -> bool {
        matches!(self, SegmentBuf::Coalesce(_))
    }
}

impl Display for SegmentBuf {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SegmentBuf::Index(i) => write!(formatter, "{}", i),
            SegmentBuf::Field(field) => write!(formatter, "{}", field),
            SegmentBuf::Coalesce(v) => write!(
                formatter,
                "({})",
                v.iter()
                    .map(|field| field.to_string())
                    .collect::<Vec<_>>()
                    .join(" | ")
            ),
        }
    }
}

impl From<String> for SegmentBuf {
    fn from(name: String) -> Self {
        Self::Field(name.into())
    }
}

impl From<&str> for SegmentBuf {
    fn from(name: &str) -> Self {
        Self::from(name.to_string())
    }
}

impl From<isize> for SegmentBuf {
    fn from(value: isize) -> Self {
        Self::index(value)
    }
}

impl From<Vec<FieldBuf>> for SegmentBuf {
    fn from(value: Vec<FieldBuf>) -> Self {
        Self::coalesce(value)
    }
}

impl<'a> From<Segment<'a>> for SegmentBuf {
    fn from(value: Segment<'a>) -> Self {
        value.as_segment_buf()
    }
}
