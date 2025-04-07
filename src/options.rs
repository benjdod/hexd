use std::ops::{Bound, RangeBounds};


/// Display options for [`Hexd`](crate::Hexd).
/// 
/// These options may be set directly, but the
/// [`HexdOptionsBuilder`] trait provides a more convenient way to fluently build
/// options off of a default or a known base set.
#[derive(Debug, Clone, Copy)]
pub struct HexdOptions {
    /// If true, any lines which are repetitions of the
    /// previous line are skipped.
    /// This is useful for large files with repeating
    /// patterns, such as binary files.
    /// If false, all lines are printed. 
    /// 
    /// ```rust
    /// use hexd::{AsHexd, options::HexdOptionsBuilder};
    /// 
    /// let v = vec![0u8; 64];
    /// 
    /// let dump = v.hexd().autoskip(true).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000000: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "*\n",
    ///     "00000030: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    /// ));
    /// 
    /// let dump = v.hexd().autoskip(false).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000000: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "00000010: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "00000020: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "00000030: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    /// ));
    /// ```
    pub autoskip: bool,

    /// If true, the hex values are printed in uppercase.
    /// Otherwise, the hex values are printed in lowercase.
    pub uppercase: bool,

    /// If true, an ASCII representation of the bytes is printed
    /// on the right side of the hex values.
    pub print_ascii: bool,

    /// If true and if combined with a [`print_range`](Self::print_range) 
    /// that does not start on an even group alignment, the hex values are 
    /// displayed offset.
    pub align: bool,

    /// The grouping options for the hex values.
    pub grouping: Grouping,

    /// The range of bytes to print.
    /// 
    /// Note: this value is typically set using [`HexdOptionsBuilder::range`].
    pub print_range: HexdRange,

    /// The offset to use for the printed index on the 
    /// left side of the hex dump.
    pub index_offset: IndexOffset
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endianness {
    BigEndian,
    LittleEndian
}

#[derive(Debug, Clone, Copy)]
pub struct HexdRange {
    pub skip: usize,
    pub limit: Option<usize>
}

impl HexdRange {
    pub fn new<R: RangeBounds<usize>>(r: R) -> Self {
        let skip = match r.start_bound() {
            Bound::Unbounded => 0usize,
            Bound::Included(s) => *s,
            Bound::Excluded(s) => s + 1
        };
        let limit = match r.end_bound() {
            Bound::Unbounded => None,
            Bound::Included(s) => Some(*s + 1),
            Bound::Excluded(s) => Some(*s)
        };

        Self { skip, limit }
    }

    pub fn length(&self) -> Option<usize> {
        self.limit.map(|lim| lim - self.skip)
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexOffset {
    Relative(usize),
    Absolute(usize)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupedOptions {
    pub group_size: GroupSize,
    pub byte_spacing: Spacing,
    pub num_groups: usize,
    pub group_spacing: Spacing
}

impl Default for GroupedOptions {
    fn default() -> Self {
        Self {
            group_size: GroupSize::Short,
            byte_spacing: Spacing::None,
            num_groups: 8,
            group_spacing: Spacing::Normal
        }
    }
}

impl GroupedOptions {
    pub fn byte_spacing(self, byte_spacing: Spacing) -> Self {
        Self { byte_spacing, ..self }
    }

    pub fn group_spacing(self, group_spacing: Spacing) -> Self {
        Self { group_spacing, ..self }
    }

    pub fn num_groups(self, num_groups: usize) -> Self {
        Self { num_groups, ..self }
    }

    pub fn group_size(self, group_size: GroupSize) -> Self {
        Self { group_size, ..self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grouping {
    Ungrouped {
        byte_count: usize,
        spacing: Spacing
    },
    Grouped(GroupedOptions)
}

impl Grouping {
    pub fn elt_width(&self) -> usize {
        match self {
            &Grouping::Ungrouped { byte_count, spacing: _ } => byte_count,
            &Grouping::Grouped(GroupedOptions { group_size, num_groups, byte_spacing: _, group_spacing: _ }) => {
                group_size.element_count() * num_groups
            }
        }
    }

    pub fn spacing_for_index(&self, index: usize) -> Spacing {
        match self {
            &Grouping::Ungrouped { byte_count: _, spacing } => spacing,
            &Grouping::Grouped(GroupedOptions { group_size, num_groups: _, byte_spacing, group_spacing }) => {
                let elt_count = group_size.element_count();
                if index % elt_count == elt_count - 1 { group_spacing } else { byte_spacing }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupSize {
    Byte,
    Short,
    Int,
    Long,
    ULong
}

impl GroupSize {
    pub fn element_count(self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Short => 2,
            Self::Int => 4,
            Self::Long => 8,
            Self::ULong => 16
        }
    }
}

/// This is used to specify the spacing between elements in a dump.
/// ```rust
/// use hexd::{AsHexd, options::{Spacing, HexdOptionsBuilder, GroupSize}};
/// 
/// let v = vec![0u8; 8];
/// 
/// assert_eq!(
///     v.hexd().ungrouped(8, Spacing::None).dump_to::<String>(),
///     "00000000: 0000000000000000 |........|\n"
/// );
/// 
/// assert_eq!(
///     v.hexd().ungrouped(8, Spacing::Normal).dump_to::<String>(),
///     "00000000: 00 00 00 00 00 00 00 00 |........|\n"
/// );
/// 
/// let v = &v[..4];
/// 
/// assert_eq!(
///     v.hexd().ungrouped(4, Spacing::Wide).dump_to::<String>(),
///     "00000000: 00  00  00  00  |....|\n"
/// );
/// 
/// assert_eq!(
///     v.hexd().ungrouped(4, Spacing::UltraWide).dump_to::<String>(),
///     "00000000: 00    00    00    00    |....|\n"
/// );
/// 
/// assert_eq!(
///     v.hexd().grouped(GroupSize::Short, Spacing::Normal, 2, Spacing::UltraWide).dump_to::<String>(),
///     "00000000: 00 00    00 00    |....|\n"
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spacing {
    /// No spacing (`""`) is included between elements.
    None,

    /// A single space (`" "`) is included between elements.
    Normal,

    /// Two spaces (`"  "`) are included between elements.
    Wide,

    /// Four spaces (`"    "`) are included between elements.
    UltraWide
}

impl Spacing {
    pub fn as_spaces(&self) -> &'static [u8] {
        match self {
            Self::None => &[],
            Self::Normal => " ".as_bytes(),
            Self::Wide => "  ".as_bytes(),
            Self::UltraWide => "    ".as_bytes()
        }
    }
}

impl Default for HexdOptions {
    fn default() -> Self {
        Self {
            autoskip: true,
            uppercase: true,
            print_ascii: true,
            align: true,
            grouping: Grouping::Grouped(GroupedOptions::default()),
            print_range: HexdRange { skip: 0, limit: None },
            index_offset: IndexOffset::Relative(0)
        }
    }
}

/// This provides a fluent API to configure options over any type
/// which holds a [`HexdOptions`] instance.
pub trait HexdOptionsBuilder: Sized {
    /// Return a new instance of `Self` with the mapping function applied
    /// to the instance's options.
    fn map_options<F: FnOnce(HexdOptions) -> HexdOptions>(self, f: F) -> Self;

    /// Return a new instance of `Self` with the given options.
    fn with_options(self, o: HexdOptions) -> Self {
        self.map_options(|_| o)
    }

    /// Set a range of bytes to dump.
    /// This is equivalent to setting the value of the [`print_range`](HexdOptions::print_range) field.
    fn range<R: RangeBounds<usize>>(self, range: R) -> Self {
        self.map_options(|o| HexdOptions {
            print_range: HexdRange::new(range),
            ..o
        })
    }
    fn aligned(self, align: bool) -> Self {
        self.map_options(|o| HexdOptions {
            align,
            ..o
        })
    }
    fn uppercase(self, uppercase: bool) -> Self {
        self.map_options(|o| HexdOptions {
            uppercase,
            ..o
        })
    }
    fn grouping(self, grouping: Grouping) -> Self {
        self.map_options(|o| HexdOptions {
            grouping,
            ..o
        })
    }
    fn ungrouped(self, num_bytes: usize, spacing: Spacing) -> Self {
        self.map_options(|o| HexdOptions {
            grouping: Grouping::Ungrouped {
                byte_count: num_bytes,
                spacing
            },
            ..o
        })
    }
    fn grouped(self, group_size: GroupSize, byte_spacing: Spacing, num_groups: usize, group_spacing: Spacing) -> Self {
        self.map_options(|o| HexdOptions {
            grouping: Grouping::Grouped(GroupedOptions { group_size, num_groups, byte_spacing, group_spacing }),
            ..o
        })
    }
    fn autoskip(self, autoskip: bool) -> Self {
        self.map_options(|o| HexdOptions {
            autoskip,
            ..o
        })
    }
    fn relative_offset(self, offset: usize) -> Self {
        self.map_options(|o| HexdOptions {
            index_offset: IndexOffset::Relative(offset),
            ..o
        })
    }
    fn absolute_offset(self, offset: usize) -> Self {
        self.map_options(|o| HexdOptions {
            index_offset: IndexOffset::Absolute(offset),
            ..o
        })
    }
}

impl HexdOptionsBuilder for HexdOptions {
    fn map_options<F: FnOnce(HexdOptions) -> HexdOptions>(self, f: F) -> Self {
        f(self)
    }
}

impl HexdOptions {
    pub fn elt_width(&self) -> usize {
        self.grouping.elt_width()
    }
}