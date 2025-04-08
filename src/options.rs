use std::ops::{Bound, RangeBounds};


/// Display options for [`Hexd`](crate::Hexd).
/// 
/// *Note: these options may be set directly, but the
/// [`HexdOptionsBuilder`] trait provides a more convenient way to fluently build
/// options off of a default or a known base set.*
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
    /// let v = vec![5u8; 64];
    /// 
    /// let dump = v.hexd().autoskip(true).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000000: 0505 0505 0505 0505 0505 0505 0505 0505 |................|\n",
    ///     "*\n",
    ///     "00000030: 0505 0505 0505 0505 0505 0505 0505 0505 |................|\n",
    /// ));
    /// 
    /// let dump = v.hexd().autoskip(false).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000000: 0505 0505 0505 0505 0505 0505 0505 0505 |................|\n",
    ///     "00000010: 0505 0505 0505 0505 0505 0505 0505 0505 |................|\n",
    ///     "00000020: 0505 0505 0505 0505 0505 0505 0505 0505 |................|\n",
    ///     "00000030: 0505 0505 0505 0505 0505 0505 0505 0505 |................|\n",
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
    /// 
    /// ```rust
    /// use hexd::{AsHexd, options::HexdOptionsBuilder};
    /// 
    /// let v = vec![0u8; 32];
    /// 
    /// let dump = v.hexd().range(7..).aligned(true).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000000:                  00 0000 0000 0000 0000 |       .........|\n",
    ///     "00000010: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n"
    /// ));
    /// 
    /// let dump = v.hexd().range(7..).aligned(false).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000007: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "00000017: 0000 0000 0000 0000 00                  |.........       |\n"
    /// ));
    /// ```
    pub align: bool,

    /// The grouping options for the hex values. For more information,
    /// see [`Grouping`].
    /// 
    /// ```
    /// use hexd::{AsHexd, options::{HexdOptionsBuilder, Spacing, GroupSize}};
    /// 
    /// let v = vec![0u8; 64];
    /// 
    /// let dump = v.hexd()
    ///     .range(..16)
    ///     .ungrouped(8, Spacing::Normal)
    ///     .dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///    "00000000: 00 00 00 00 00 00 00 00 |........|\n",
    ///    "00000008: 00 00 00 00 00 00 00 00 |........|\n",
    /// ));
    /// 
    /// let dump = v.hexd()
    ///     .range(..16)
    ///     .grouped(GroupSize::Short, Spacing::Normal, 4, Spacing::Wide)
    ///     .dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///    "00000000: 00 00  00 00  00 00  00 00  |........|\n",
    ///    "00000008: 00 00  00 00  00 00  00 00  |........|\n",
    /// ));
    /// 
    /// let dump = v.hexd()
    ///     .range(..32)
    ///     .grouped(GroupSize::Int, Spacing::None, 4, Spacing::Normal)
    ///     .dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///    "00000000: 00000000 00000000 00000000 00000000 |................|\n",
    ///    "00000010: 00000000 00000000 00000000 00000000 |................|\n",
    /// ));
    /// ```
    pub grouping: Grouping,

    /// The range of bytes to print.
    /// 
    /// ```
    /// use hexd::{AsHexd, options::HexdOptionsBuilder};
    /// 
    /// let v = vec![0u8; 256];
    /// 
    /// let dump = v.hexd().range(0x47..0xb3).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000040:                  00 0000 0000 0000 0000 |       .........|\n",
    ///     "00000050: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "*\n",
    ///     "000000A0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "000000B0: 0000 00                                 |...             |\n",
    /// ));
    /// ```
    pub print_range: HexdRange,

    /// The offset to use for the printed index on the 
    /// left side of the hex dump.
    /// 
    /// ```
    /// use hexd::{AsHexd, options::HexdOptionsBuilder};
    /// 
    /// let v = vec![0u8; 256];
    /// 
    /// let dump = v.hexd().range(0x47..0xb3).relative_offset(0xfff0000).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "0FFF0040:                  00 0000 0000 0000 0000 |       .........|\n",
    ///     "0FFF0050: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "*\n",
    ///     "0FFF00A0: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "0FFF00B0: 0000 00                                 |...             |\n",
    /// ));
    /// 
    /// let v = &v[..64];
    /// let dump = v.hexd().absolute_offset(0x201B).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "0000201B: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    ///     "*\n",
    ///     "0000204B: 0000 0000 0000 0000 0000 0000 0000 0000 |................|\n",
    /// ));
    /// ```
    pub index_offset: IndexOffset,

    /// Flush behavior to use when writing the hexdump. 
    /// 
    /// *Note: this is likely only useful when writing to a stream or IO-based output.*
    pub flush: FlushMode
}

/// Control how often [`flush`](method@crate::writer::WriteHexdump::flush) is called on the writer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlushMode {
    /// Call [`flush`](method@crate::writer::WriteHexdump::flush) on the writer after `n` lines
    /// have been written to it.
    AfterNLines(usize),

    /// Call [`flush`](method@crate::writer::WriteHexdump::flush) on the writer after all lines
    /// have been written to it.
    End
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endianness {
    BigEndian,
    LittleEndian
}

#[derive(Debug, Clone, Copy)]
pub struct HexdRange {
    /// The number of bytes to skip before printing.
    pub skip: usize,
    /// The number of bytes to print.
    pub limit: Option<usize>
}

impl HexdRange {
    /// Return a new instance which includes all bytes.
    pub fn full() -> Self {
        Self { skip: 0, limit: None }
    }
    
    /// Return a new instance which includes the bytes specified by the range.
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

    /// If [`limit`](field@HexdRange::limit) is not None, return the length
    /// of the range.
    pub fn length(&self) -> Option<usize> {
        self.limit.map(|lim| lim - self.skip)
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexOffset {
    Relative(usize),
    Absolute(usize)
}

/// This trait controls how bytes are grouped in the hexdump.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Grouping {
    /// No grouping is applied. A total of `byte_count` bytes
    /// are printed with a constant amount of [`spacing`](Spacing) between them.
    /// 
    /// ```
    /// use hexd::{AsHexd, options::{Spacing, HexdOptionsBuilder}};
    /// 
    /// let v = vec![0u8; 16];
    /// let dump = v.hexd().ungrouped(8, Spacing::None).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000000: 0000000000000000 |........|\n",
    ///     "00000008: 0000000000000000 |........|\n",
    /// ));
    /// 
    /// let v = vec![0u8; 16];
    /// let dump = v.hexd().ungrouped(8, Spacing::Normal).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000000: 00 00 00 00 00 00 00 00 |........|\n",
    ///     "00000008: 00 00 00 00 00 00 00 00 |........|\n",
    /// ));
    /// let v = vec![0u8; 8];
    /// let dump = v.hexd().ungrouped(4, Spacing::Wide).dump_to::<String>();
    /// assert_eq!(dump, concat!(
    ///     "00000000: 00  00  00  00  |....|\n",
    ///     "00000004: 00  00  00  00  |....|\n",
    /// ));
    /// ```
    Ungrouped {
        byte_count: usize,
        spacing: Spacing
    },

    /// The bytes are grouped into `num_groups` of `group_size` bytes each.
    /// The spacing between the bytes in a group is `byte_spacing`,
    /// and the spacing between groups is `group_spacing`.
    Grouped {
        group_size: GroupSize,
        byte_spacing: Spacing,
        num_groups: usize,
        group_spacing: Spacing
    }
}

impl Grouping {
    pub fn elt_width(&self) -> usize {
        match self {
            &Grouping::Ungrouped { byte_count, spacing: _ } => byte_count,
            &Grouping::Grouped { group_size, num_groups, byte_spacing: _, group_spacing: _ } => {
                group_size.element_count() * num_groups
            }
        }
    }

    pub fn spacing_for_index(&self, index: usize) -> Spacing {
        match self {
            &Grouping::Ungrouped { byte_count: _, spacing } => spacing,
            &Grouping::Grouped { group_size, num_groups: _, byte_spacing, group_spacing } => {
                let elt_count = group_size.element_count();
                if index % elt_count == elt_count - 1 { group_spacing } else { byte_spacing }
            }
        }
    }
}

impl Default for Grouping {
    fn default() -> Self {
        Self::Grouped {
            group_size: GroupSize::Short,
            byte_spacing: Spacing::None,
            num_groups: 8,
            group_spacing: Spacing::Normal
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

/// The default options for [`Hexd`](crate::Hexd).
/// 
/// ```rust,ignore
/// HexdOptions {
///     autoskip: true,
///     uppercase: true,
///     print_ascii: true,
///     align: true,
///     grouping: Grouping::default(),
///     print_range: HexdRange { skip: 0, limit: None },
///     index_offset: IndexOffset::Relative(0)
/// }
/// ```
impl Default for HexdOptions {
    fn default() -> Self {
        Self {
            autoskip: true,
            uppercase: true,
            print_ascii: true,
            align: true,
            grouping: Grouping::default(),
            print_range: HexdRange { skip: 0, limit: None },
            index_offset: IndexOffset::Relative(0),
            flush: FlushMode::End
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
    /// Set the value of the [`align`](HexdOptions::align) field.
    fn aligned(self, align: bool) -> Self {
        self.map_options(|o| HexdOptions {
            align,
            ..o
        })
    }

    /// Set the value of the [`uppercase`](HexdOptions::uppercase) field.
    fn uppercase(self, uppercase: bool) -> Self {
        self.map_options(|o| HexdOptions {
            uppercase,
            ..o
        })
    }

    /// Set the value of the [`grouping`](field@HexdOptions::grouping) field.
    fn grouping(self, grouping: Grouping) -> Self {
        self.map_options(|o| HexdOptions {
            grouping,
            ..o
        })
    }

    /// Set the value of the [`grouping`](field@HexdOptions::grouping) field to [`Grouping::Ungrouped`]
    /// using the specified parameters.
    fn ungrouped(self, num_bytes: usize, spacing: Spacing) -> Self {
        self.map_options(|o| HexdOptions {
            grouping: Grouping::Ungrouped {
                byte_count: num_bytes,
                spacing
            },
            ..o
        })
    }

    /// Set the value of the [`grouping`](field@HexdOptions::grouping) field to [`Grouping::Grouped`]
    /// using the specified parameters.
    fn grouped(self, group_size: GroupSize, byte_spacing: Spacing, num_groups: usize, group_spacing: Spacing) -> Self {
        self.map_options(|o| HexdOptions {
            grouping: Grouping::Grouped { group_size, num_groups, byte_spacing, group_spacing },
            ..o
        })
    }


    /// Set the value of the [`autoskip`](HexdOptions::autoskip) field.
    fn autoskip(self, autoskip: bool) -> Self {
        self.map_options(|o| HexdOptions {
            autoskip,
            ..o
        })
    }

    /// Set the value of the [`index_offset`](HexdOptions::index_offset) field.
    fn offset(self, index_offset: IndexOffset) -> Self {
        self.map_options(|o| HexdOptions {
            index_offset,
            ..o
        })
    }

    /// Set the value of the [`index_offset`](HexdOptions::index_offset) field to [`IndexOffset::Relative`].
    fn relative_offset(self, offset: usize) -> Self {
        self.map_options(|o| HexdOptions {
            index_offset: IndexOffset::Relative(offset),
            ..o
        })
    }

    /// Set the value of the [`index_offset`](HexdOptions::index_offset) field to [`IndexOffset::Absolute`].
    fn absolute_offset(self, offset: usize) -> Self {
        self.map_options(|o| HexdOptions {
            index_offset: IndexOffset::Absolute(offset),
            ..o
        })
    }

    /// Set the value of the [`flush`](HexdOptions::flush) field.
    fn flush(self, flush: FlushMode) -> Self {
        self.map_options(|o| HexdOptions {
            flush,
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