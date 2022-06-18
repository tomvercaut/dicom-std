use std::cmp::Ordering;
use std::collections::btree_map::BTreeMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{IodModuleTypeError, TagError, TagRangeError, VMError, VRError};

/// Reference to a node in an XML document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// unique identifier in the XML document
    pub target: String,
    /// text render selector
    pub style: String,
}

impl Default for Link {
    fn default() -> Self {
        Self {
            target: "".to_string(),
            style: "".to_string(),
        }
    }
}

impl std::fmt::Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} [style={}]",
            self.target.as_str(),
            self.style.as_str()
        )
    }
}

/// Usage of a Module or Functional group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Usage {
    None,
    /// Mandatory
    M,
    /// User option
    U,
    /// Conditional
    C,
}

impl Default for Usage {
    fn default() -> Self {
        Self::None
    }
}

impl std::fmt::Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Usage::None => {
                write!(f, "None")
            }
            Usage::M => {
                write!(f, "M")
            }
            Usage::U => {
                write!(f, "U")
            }
            Usage::C => {
                write!(f, "C")
            }
        }
    }
}

impl<'a> From<&'a str> for Usage {
    fn from(s: &'a str) -> Self {
        let n = s.len();
        if n > 0 {
            let c = s.chars().next().unwrap();
            if c == 'M' {
                return Usage::M;
            } else if c == 'U' {
                return Usage::U;
            } else if c == 'C' {
                return Usage::C;
            }
        }
        Usage::None
    }
}

/// Stores the name of a Module, a Reference to the details of the module defintion in the
/// XML document and the usage of the module definition in the Information Entity(IE).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeModuleReferenceUsage {
    /// name of the module
    pub module: String,
    /// reference to the module details in the XML document
    pub reference: Link,
    /// usage requirement in the information entity
    pub usage: Usage,
}

impl Default for CompositeModuleReferenceUsage {
    fn default() -> Self {
        Self {
            module: "".to_string(),
            reference: Default::default(),
            usage: Default::default(),
        }
    }
}

/// An Information Entity item in a composite IOD module table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeIodModuleItem {
    /// name of the Information Entity item
    pub ie: String,
    /// a list of IE items storing the module, reference and usage
    pub items: Vec<CompositeModuleReferenceUsage>,
    /// number items that are expected,
    /// this value should only be during parsing for validation purposes
    pub expected_items: usize,
}

impl Default for CompositeIodModuleItem {
    fn default() -> Self {
        Self {
            ie: "".to_string(),
            items: vec![],
            expected_items: 0,
        }
    }
}

/// A composite IOD module
///
/// See [Composite Information Object Definitions](https://dicom.nema.org/medical/dicom/current/output/html/part03.html#chapter_A)
/// in the DICOM standard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeIodModule {
    /// title of the module
    pub caption: String,
    /// list of module items
    pub items: Vec<CompositeIodModuleItem>,
}

impl Default for CompositeIodModule {
    fn default() -> Self {
        Self {
            caption: "".to_string(),
            items: vec![],
        }
    }
}

/// DICOM tag consisting of a group and element.
#[derive(Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    /// group
    pub g: u16,
    /// element
    pub e: u16,
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Option::from(match self.g.cmp(&other.g) {
            Ordering::Equal => self.e.cmp(&other.e),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        })
        // if self.g == other.g {
        //     return if self.e == other.e {
        //         Some(Ordering::Equal)
        //     } else if self.e < other.e {
        //         Some(Ordering::Less)
        //     } else {
        //         Some(Ordering::Greater)
        //     };
        // } else if self.g < other.g {
        //     return Some(Ordering::Less);
        // }
        // Some(Ordering::Greater)
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // # adds 0x in formatting
        // let sg = format!("{:#04X}", self.g);
        let sg = format!("{:04X}", self.g);
        let se = format!("{:04X}", self.e);
        write!(f, "({},{})", &sg, &se)
    }
}

impl Eq for Tag {}

/// Create a Tag from a string.
///
/// Expected string format: (1234,5678)
/// * group value: 1234 (=> 0x1234)
/// * element value: 5678 (=> 0x5678)
impl FromStr for Tag {
    type Err = TagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ls = s.to_lowercase();
        let mut ts = ls.trim_matches(is_char_whitespace_or_return);
        if let Some(x) = ts.find('(') {
            if x >= ts.len() - 1 {
                return Err(TagError::InvalidFormat(s.to_string()));
            }
            ts = &ts[x + 1..];
        }
        if let Some(x) = ts.find(')') {
            ts = &ts[0..x];
        }
        let pos = ts.find(',');
        if pos.is_none() {
            return Err(TagError::InvalidFormat(s.to_string()));
        }
        let pos = pos.unwrap();
        if pos >= ts.len() - 1 {
            return Err(TagError::InvalidFormat(s.to_string()));
        }
        let sg = &ts[0..pos];
        let se = &ts[pos + 1..];
        if se.contains(',') {
            return Err(TagError::InvalidFormat(ts.to_string()));
        }

        let to_u16 = |t: &str| -> Result<u16, Self::Err> {
            let n = t.len();
            if let Some(x) = t.find('x') {
                if x >= n - 1 {
                    return Err(TagError::InvalidFormat(s.to_string()));
                }
                u16::from_str_radix(&t[x + 1..], 16)
                    .map_err(|e| TagError::ParseIntError { source: e })
            } else {
                u16::from_str_radix(t, 16).map_err(|e| TagError::ParseIntError { source: e })
            }
        };
        let g = to_u16(sg)?;
        let e = to_u16(se)?;
        Ok(Self { g, e })
    }
}

impl Tag {
    /// Create a new Tag value from group and element integers.
    ///
    /// # Arguments
    /// * `g` - group value
    /// * `e` - element value
    pub fn new(g: u16, e: u16) -> Self {
        Self { g, e }
    }
    /// Convert a Tag into a unsigned 32 bit integer by combining the group and element values.
    pub fn as_u32(&self) -> u32 {
        (((self.g as u32) << 16) & 0xFFFF0000) | ((self.e as u32) & 0x0000FFFF)
    }

    /// Check if the group in DICOM is valid. Group numbers 1,3, 5, 7 and 0xFFFF are illegal.
    pub fn has_valid_group(&self) -> bool {
        if ((self.g & 1) != 0) && (self.g <= 7 || self.g == 0xFFFF) {
            return false;
        }
        true
    }

    /// Is the tag private?
    pub fn is_private(&self) -> bool {
        (self.g & 1) != 0 && self.has_valid_group()
    }

    /// Check if the tag is a private reservation tag (GGGG,00xx) where GGGG is odd and
    /// xx is in the range [0x10 - 0xFF]
    pub fn is_private_reservation(&self) -> bool {
        self.is_private() && self.e >= 0x0010 && self.e <= 0x00FF
    }
}

/// DICOM tag range implementation as defined in the
/// [Conventions](https://dicom.nema.org/medical/dicom/current/output/html/part06.html#chapter_5)
/// in the DICOM PS3.6 Data Dictionary.
#[derive(Copy, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TagRange {
    /// Lowest Tag value in the tag range.
    min: Tag,
    /// Highest Tag value in the tag range.
    max: Tag,
    /// True if the low and high differ.
    is_range: bool,
}

impl std::fmt::Display for TagRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if !self.is_range {
            write!(f, "{}", self.min)
        } else {
            let s1 = format!("{}", self.min);
            let s2 = format!("{}", self.max);
            let n = s1.len();
            assert_eq!(
                n,
                s2.len(),
                "expected both tag [{} <-> {}] to have the same string length",
                s1,
                s2
            );
            let mut s = String::with_capacity(n);
            for i in 0..n {
                if s1.chars().nth(i).unwrap() != s2.chars().nth(i).unwrap() {
                    s.push('x');
                } else {
                    let c = s1.chars().nth(i).unwrap();
                    s.push(c);
                }
            }
            write!(f, "{}", s)
        }
    }
}

impl FromStr for TagRange {
    type Err = TagRangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.to_lowercase();
        let smin = t.replace('x', "0");
        let smax = t.replace('x', "F");
        let min = Tag::from_str(smin.as_str())?;
        let max = Tag::from_str(smax.as_str())?;
        Ok(TagRange::new(min, max))
    }
}

impl From<Tag> for TagRange {
    fn from(tag: Tag) -> Self {
        Self {
            min: tag,
            max: tag,
            is_range: false,
        }
    }
}

impl PartialEq<Self> for TagRange {
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}

impl Eq for TagRange {}

impl PartialOrd<Self> for TagRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.min.partial_cmp(&other.min)
    }
}

impl Ord for TagRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.min.cmp(&other.min)
    }
}

impl TagRange {
    /// Create a new TagRange.
    ///
    /// # Arguments
    /// * `min` - lowest Tag value in the range
    /// * `max` - highest Tag value in the range
    ///
    pub fn new(min: Tag, max: Tag) -> Self {
        if min == max {
            Self {
                min,
                max,
                is_range: false,
            }
        } else {
            Self {
                min,
                max,
                is_range: true,
            }
        }
    }

    /// Get the lowest Tag value in the range.
    #[inline]
    pub fn min(&self) -> &Tag {
        &self.min
    }

    /// Set the lowest Tag value in the range.
    ///
    /// # Arguments
    /// * `tag` - Tag value
    pub fn set_min(&mut self, tag: Tag) {
        self.min = tag;
        if self.min != self.max {
            self.is_range = true;
        } else {
            self.is_range = false;
        }
    }

    /// Get the highest Tag value in the range.
    #[inline]
    pub fn max(&self) -> &Tag {
        &self.max
    }

    /// Set the highest Tag value in the range.
    ///
    /// # Arguments
    /// * `tag` - Tag value
    pub fn set_max(&mut self, tag: Tag) {
        self.max = tag;
        if self.min != self.max {
            self.is_range = true;
        } else {
            self.is_range = false;
        }
    }

    /// True if the lowest and highest Tag value are different.
    #[inline]
    pub fn is_range(&self) -> bool {
        self.is_range
    }

    /// Check if a tag is equal to or within bounds of the tag range.
    ///
    /// # Arguments
    /// * `tag` - DICOM tag
    #[inline]
    pub fn contains(&self, tag: &Tag) -> bool {
        self.min.g <= tag.g && tag.g <= self.max.g && self.min.e <= tag.e && tag.e <= self.max.e
    }
}

/// DICOM Attribute Requirement Types
/// - Type 1: Required to be in the SOP Instance and shall have a valid value.
/// - Type 2: Required to be in the SOP Instance but may contain the value of "unknown", or a zero length value.
/// - Type 3: Optional. May or may not be included and could be zero length.
/// - Type 1C: Conditional. If a condition is met, then it is a Type 1 (required, cannot be zero). If condition is not met, then the tag is not sent.
/// - Type 2C: Conditional. If condition is met, then it is a Type 2 (required, zero length OK). If condition is not met, then the tag is not sent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IodModuleType {
    None,
    /// Type 1: Required to be in the SOP Instance and shall have a valid value.
    One,
    /// Type 1C: Conditional. If a condition is met, then it is a Type 1 (required, cannot be zero). If condition is not met, then the tag is not sent.
    OneC,
    /// Type 2: Required to be in the SOP Instance but may contain the value of "unknown", or a zero length value.
    Two,
    /// Type 2C: Conditional. If condition is met, then it is a Type 2 (required, zero length OK). If condition is not met, then the tag is not sent.
    TwoC,
    /// Type 3: Optional. May or may not be included and could be zero length.
    Three,
}

impl Default for IodModuleType {
    fn default() -> Self {
        Self::None
    }
}

impl FromStr for IodModuleType {
    type Err = IodModuleTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "1" {
            Ok(Self::One)
        } else if s == "1C" {
            Ok(Self::OneC)
        } else if s == "2" {
            Ok(Self::Two)
        } else if s == "2C" {
            Ok(Self::TwoC)
        } else if s == "3" {
            Ok(Self::Three)
        } else {
            Err(IodModuleTypeError::InvalidFormat(s.to_string()))
        }
    }
}

/// Module definition attribute models an item in Information Module Definition table.
///
/// See [Information Module Definitions](https://dicom.nema.org/medical/dicom/current/output/html/part03.html#chapter_C) in the DICOM standard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleAttribute {
    /// identifying attribute name
    pub name: String,
    /// DICOM tag uniquely identifying the attribute
    pub tag: Tag,
    /// Type indicating if the attribute is required in a Module.
    pub type_: IodModuleType,
    /// Additional information on the attribute
    pub description: String,
}

impl Default for ModuleAttribute {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            tag: Default::default(),
            type_: Default::default(),
            description: "".to_string(),
        }
    }
}

/// An include item in Information Module Definition table.
///
/// See [Information Module Definitions](https://dicom.nema.org/medical/dicom/current/output/html/part03.html#chapter_C) in the DICOM standard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleAttributeInclude {
    /// Include statement with the indication of the depth of a sequence
    pub text: String,
    /// Link to a unique identifier in the XML document
    pub link: Link,
    /// Additional information on the include attribute
    pub description: String,
}

impl Default for ModuleAttributeInclude {
    fn default() -> Self {
        Self {
            text: "".to_string(),
            link: Default::default(),
            description: "".to_string(),
        }
    }
}

/// An attribute that indicates that the items within are the same onces as referenced in the top level attribute.
///
/// For an example of this can be found in the DICOM standard:
/// [Original Attributes Macro Attributes](https://dicom.nema.org/medical/dicom/current/output/html/part03.html#table_C.12.1.1.9-1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleAttributeReferenceTopLevelAttributes {
    /// Statement indicating the attribute is a reference to attributes at the top level data set.
    pub text: String,
    /// Type indicating if the attribute is required in the Module
    pub type_: IodModuleType,
    /// Additional information on the attribute
    pub description: String,
}

impl Default for ModuleAttributeReferenceTopLevelAttributes {
    fn default() -> Self {
        Self {
            text: "".to_string(),
            type_: Default::default(),
            description: "".to_string(),
        }
    }
}

/// Different type of possible items in a Module definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModuleDefinitionItem {
    None,
    Module(ModuleAttribute),
    Macro(ModuleAttribute),
    Include(ModuleAttributeInclude),
    TopLevelAttributes(ModuleAttributeReferenceTopLevelAttributes),
}

impl Default for ModuleDefinitionItem {
    fn default() -> Self {
        Self::None
    }
}

/// Module defintion as described in the
/// [DICOM standard](https://dicom.nema.org/medical/dicom/current/output/html/part03.html#chapter_C).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleDefinition {
    /// unique identifier in the XML document
    pub id: String,
    /// Module definition title
    pub caption: String,
    /// Module definition items
    pub items: Vec<ModuleDefinitionItem>,
}

impl Default for ModuleDefinition {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            caption: "".to_string(),
            items: vec![],
        }
    }
}

/// Test if a character is a whitespace, a carriage return or a newline.
///
/// # Arguments
///
/// * `c` - character
#[inline]
pub(crate) fn is_char_whitespace_or_return(c: char) -> bool {
    c.is_whitespace() || c == '\r' || c == '\n'
}

/// The Information Object Definition(IOD) library.
///
/// Within this library the following items are defined:
/// * a list of composite IODs modules,
/// * a map linking a unique IOD module name with one or more XML table IDs
/// * a map linking the unique XML table IDs with a corresponding ModuleDefinition
///
/// The IODLibrary contains the necessary data to build a full model to represent
/// composite IOD modules and their individual DICOM entries.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct IODLibrary {
    /// List of composite IOD modules
    pub composite: Vec<CompositeIodModule>,
    /// Key: module name, Values: list of table IDs belonging to the corresponding module
    pub module_tables_ids: BTreeMap<String, Vec<String>>,
    /// List of normative module definitions
    pub normative: BTreeMap<String, ModuleDefinition>,
}

/// Stores all the data parsed from the DICOM standard.
///
/// This object doesn't contain all the data in the standard but only a small subset:
/// * library of composite IODs and module definitions
/// * dictionary of DICOM tag(ranges)s
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DicomStandard {
    pub iod_lib: IODLibrary,
    pub dict: DataDictionary,
}

/// Value Multiplicity (VM) specifies how many values can be encoded in the value field of a DICOM data element.
///
/// More information on VM can be found in the [part 5, section 6.4 of the DICOM standard](https://dicom.nema.org/medical/dicom/current/output/html/part05.html#sect_6.4)
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VM {
    /// Minimum number of values that are encoded.
    pub min: u16,
    /// Maximum number of values that are encoded.
    pub max: u16,
    /// True if the minimum number of values is defined in the standard by `n` (e.g. 2n).
    pub is_min_n: bool,
    /// True if the maximum number of values is defined in the standard by `n` (e.g. 2n).
    pub is_max_n: bool,
}

impl std::fmt::Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min > 0 {
            write!(f, "{}", self.min)?;
        }
        if self.is_min_n {
            write!(f, "n")?;
        }
        if self.max > 0 || self.is_max_n {
            write!(f, "-")?;
            if self.max > 0 {
                write!(f, "{}", self.max)?;
            }
            if self.is_max_n {
                write!(f, "n")?;
            }
        }
        Ok(())
    }
}

/// Create a VM from a string. The expected formats are:
/// * a single integer: e.g. 1
/// * a range of integers: e.g.
///     * 2-10 where `2` is the minimum amount of encoded values and `10` the maximum.
///     * 2-n where `2` is the minimum amount of encoded values and `n` the maximum.
///     * 2-4n where `2` is the minimum amount of encoded values and `4n` the maximum.
///     * 2n-4n where `2n` is the minimum amount of encoded values and `4n` the maximum.
///
impl FromStr for VM {
    type Err = VMError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let f = |t: &str| -> Result<(u16, bool), Self::Err> {
            match t.find('n') {
                None => Ok((u16::from_str(t)?, false)),
                Some(pos) => {
                    if pos == 0 {
                        Ok((0, true))
                    } else {
                        Ok((u16::from_str(&t[0..pos])?, true))
                    }
                }
            }
        };
        let mut vm = VM::default();
        match s.find('-') {
            None => {
                let (x, b) = f(s)?;
                vm.min = x;
                vm.is_min_n = b;
                Ok(vm)
            }
            Some(pos) => {
                {
                    let (x, b) = f(&s[0..pos])?;
                    vm.min = x;
                    vm.is_min_n = b;
                }
                {
                    let (x, b) = f(&s[pos + 1..])?;
                    vm.max = x;
                    vm.is_max_n = b;
                }
                Ok(vm)
            }
        }
    }
}

macro_rules! impl_from_primitive_for_vm {
    ($typ: ty) => {
        impl From<$typ> for VM {
            fn from(value: $typ) -> Self {
                Self {
                    min: value as u16,
                    max: value as u16,
                    is_min_n: false,
                    is_max_n: false,
                }
            }
        }
    };
}

impl_from_primitive_for_vm!(i8);
impl_from_primitive_for_vm!(i16);
impl_from_primitive_for_vm!(i32);
impl_from_primitive_for_vm!(i64);
impl_from_primitive_for_vm!(isize);

impl_from_primitive_for_vm!(u8);
impl_from_primitive_for_vm!(u16);
impl_from_primitive_for_vm!(u32);
impl_from_primitive_for_vm!(u64);
impl_from_primitive_for_vm!(usize);

/// Value Representation provides information on the type of data that stored in a DICOM data element.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VR {
    AE,
    AS,
    AT,
    CS,
    DA,
    DS,
    DT,
    FL,
    FD,
    IS,
    LO,
    LT,
    OB,
    OD,
    OF,
    OL,
    OV,
    OW,
    PN,
    SH,
    SL,
    SQ,
    SS,
    ST,
    SV,
    TM,
    UC,
    UI,
    UL,
    UN,
    UR,
    US,
    UT,
    UV,
}

impl FromStr for VR {
    type Err = VRError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "AE" {
            Ok(VR::AE)
        } else if s == "AS" {
            Ok(VR::AS)
        } else if s == "AT" {
            Ok(VR::AT)
        } else if s == "CS" {
            Ok(VR::CS)
        } else if s == "DA" {
            Ok(VR::DA)
        } else if s == "DS" {
            Ok(VR::DS)
        } else if s == "DT" {
            Ok(VR::DT)
        } else if s == "FL" {
            Ok(VR::FL)
        } else if s == "FD" {
            Ok(VR::FD)
        } else if s == "IS" {
            Ok(VR::IS)
        } else if s == "LO" {
            Ok(VR::LO)
        } else if s == "LT" {
            Ok(VR::LT)
        } else if s == "OB" {
            Ok(VR::OB)
        } else if s == "OD" {
            Ok(VR::OD)
        } else if s == "OF" {
            Ok(VR::OF)
        } else if s == "OL" {
            Ok(VR::OL)
        } else if s == "OV" {
            Ok(VR::OV)
        } else if s == "OW" {
            Ok(VR::OW)
        } else if s == "PN" {
            Ok(VR::PN)
        } else if s == "SH" {
            Ok(VR::SH)
        } else if s == "SL" {
            Ok(VR::SL)
        } else if s == "SQ" {
            Ok(VR::SQ)
        } else if s == "SS" {
            Ok(VR::SS)
        } else if s == "ST" {
            Ok(VR::ST)
        } else if s == "SV" {
            Ok(VR::SV)
        } else if s == "TM" {
            Ok(VR::TM)
        } else if s == "UC" {
            Ok(VR::UC)
        } else if s == "UI" {
            Ok(VR::UI)
        } else if s == "UL" {
            Ok(VR::UL)
        } else if s == "UN" {
            Ok(VR::UN)
        } else if s == "UR" {
            Ok(VR::UR)
        } else if s == "US" {
            Ok(VR::US)
        } else if s == "UT" {
            Ok(VR::UT)
        } else if s == "UV" {
            Ok(VR::UV)
        } else {
            Err(VRError::InvalidFormat(s.to_string()))
        }
    }
}

impl std::fmt::Display for VR {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VR::AE => {
                write!(f, "AE")
            }
            VR::AS => {
                write!(f, "AS")
            }
            VR::AT => {
                write!(f, "AT")
            }
            VR::CS => {
                write!(f, "CS")
            }
            VR::DA => {
                write!(f, "DA")
            }
            VR::DS => {
                write!(f, "DS")
            }
            VR::DT => {
                write!(f, "DT")
            }
            VR::FL => {
                write!(f, "FL")
            }
            VR::FD => {
                write!(f, "FD")
            }
            VR::IS => {
                write!(f, "IS")
            }
            VR::LO => {
                write!(f, "LO")
            }
            VR::LT => {
                write!(f, "LT")
            }
            VR::OB => {
                write!(f, "OB")
            }
            VR::OD => {
                write!(f, "OD")
            }
            VR::OF => {
                write!(f, "OF")
            }
            VR::OL => {
                write!(f, "OL")
            }
            VR::OV => {
                write!(f, "OV")
            }
            VR::OW => {
                write!(f, "OW")
            }
            VR::PN => {
                write!(f, "PN")
            }
            VR::SH => {
                write!(f, "SH")
            }
            VR::SL => {
                write!(f, "SL")
            }
            VR::SQ => {
                write!(f, "SQ")
            }
            VR::SS => {
                write!(f, "SS")
            }
            VR::ST => {
                write!(f, "ST")
            }
            VR::SV => {
                write!(f, "SV")
            }
            VR::TM => {
                write!(f, "TM")
            }
            VR::UC => {
                write!(f, "UC")
            }
            VR::UI => {
                write!(f, "UI")
            }
            VR::UL => {
                write!(f, "UL")
            }
            VR::UN => {
                write!(f, "UN")
            }
            VR::UR => {
                write!(f, "UR")
            }
            VR::US => {
                write!(f, "US")
            }
            VR::UT => {
                write!(f, "UT")
            }
            VR::UV => {
                write!(f, "UV")
            }
        }
    }
}

/// Entry in the registry of the DICOM data elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataDictionaryEntry {
    /// DICOM tag (0010,0020) or tag range (7Fxx,0030)
    pub tag: TagRange,
    /// Name of the DICOM tag
    pub name: String,
    /// Keyword of the DICOM tag (doesn't contain spaces)
    pub keyword: String,
    /// Value Representation
    pub vr: Vec<VR>,
    /// Value Multiplicity
    pub vm: VM,
    /// Description of the DICOM tag
    pub description: String,
    /// True if the tag is retired.
    pub retired: bool,
}

impl Default for DataDictionaryEntry {
    fn default() -> Self {
        Self {
            tag: TagRange::default(),
            name: "".to_string(),
            keyword: "".to_string(),
            vr: vec![],
            vm: VM::default(),
            description: "".to_string(),
            retired: false,
        }
    }
}

impl DataDictionaryEntry {
    /// Check if a DICOM tag is a sequence (Value Representation is SQ).
    pub fn is_seq(&self) -> bool {
        for vr in &self.vr {
            if vr == &VR::SQ {
                return true;
            }
        }
        false
    }

    /// Check if a DICOM tag is retired.
    pub fn is_retired(&self) -> bool {
        self.retired
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataDictionary {
    data: BTreeMap<TagRange, DataDictionaryEntry>,
}

impl DataDictionary {
    /// Find a tag in the data dictionary by tag.
    ///
    /// # Arguments
    /// * `tag` - DICOM tag
    pub fn by_tag(&self, tag: Tag) -> Option<&DataDictionaryEntry> {
        let tr = TagRange::from(tag);
        self.data.get(&tr)
    }

    /// Check if the data dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Add an entry to the data dictionary.
    ///
    /// # Arguments
    /// * `entry` - DICOM data dictionary entry
    pub fn add(&mut self, entry: DataDictionaryEntry) -> bool {
        if self.data.contains_key(&entry.tag) {
            return false;
        }
        self.data.insert(entry.tag, entry);
        true
    }

    pub fn extend(&mut self, other: &mut DataDictionary) {
        let keys: Vec<TagRange> = other.data.keys().cloned().collect();
        for k in keys {
            {
                let entry = other.data.get(&k).unwrap();
                self.data.insert(k, entry.clone());
            }
            other.data.remove(&k);
        }
    }

    /// Remove all entries in the data dictionary.
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Tag, TagRange, VM};
    use std::str::FromStr;

    #[test]
    fn tag_from_str() {
        let vs = [
            "0102,3252",
            "0x0102,0x3252",
            "0102,0x3252",
            "(0x0102,3252)",
            "0x0102,3252",
        ];
        let vge = [
            (0x0102_u16, 0x3252_u16),
            (0x0102, 0x3252),
            (0x0102, 0x3252),
            (0x0102, 0x3252),
            (0x0102, 0x3252),
        ];
        assert_eq!(vs.len(), vge.len());
        for (s, (g, e)) in vs.iter().zip(vge.iter()) {
            let tag = Tag::from_str(s);
            assert!(tag.is_ok());
            let tag = tag.unwrap();
            assert_eq!(tag.g, *g);
            assert_eq!(tag.e, *e);
        }
    }

    #[test]
    fn tag_display() {
        let arr1 = [Tag::default()];
        let arr2 = ["(0000,0000)"];
        for (a, b) in arr1.iter().zip(arr2.iter()) {
            let c = a.to_string();
            assert_eq!(c.as_str(), *b);
        }
    }

    #[test]
    fn tag_to_u32() {
        let arr1 = [Tag {
            g: 0x6789,
            e: 0x2345,
        }];
        let arr2 = [0x67892345_u32];
        for (a, b) in arr1.iter().zip(arr2.iter()) {
            let x = a.as_u32();
            assert_eq!(x, *b);
        }
    }

    #[test]
    fn tag_range_display() {
        let arr = [
            TagRange::from_str("(5372,1234)").unwrap(),
            TagRange::from_str("(53xx,1234)").unwrap(),
            TagRange::from_str("(5372,12xx)").unwrap(),
        ];
        let e_arr = ["(5372,1234)", "(53xx,1234)", "(5372,12xx)"];
        assert_eq!(arr.len(), e_arr.len());
        for (a, e) in arr.iter().zip(e_arr.iter()) {
            let s = a.to_string();
            assert_eq!(s.as_str(), *e);
        }
    }

    #[test]
    fn tag_range_max() {
        let arr1 = [
            TagRange::from_str("6789,1234").unwrap(),
            TagRange::from_str("67xx,1234").unwrap(),
            TagRange::from_str("6789,12xx").unwrap(),
        ];
        let arr2 = [
            Tag {
                g: 0x6789,
                e: 0x1234,
            },
            Tag {
                g: 0x67FF,
                e: 0x1234,
            },
            Tag {
                g: 0x6789,
                e: 0x12FF,
            },
        ];
        assert_eq!(arr1.len(), arr2.len());
        for (a, b) in arr1.iter().zip(arr2.iter()) {
            let u = a.max();
            assert_eq!(*u, *b);
        }
    }

    #[test]
    fn tag_range_contains() {
        let arr1 = [
            TagRange::from_str("(7FE0,0030)").unwrap(),
            TagRange::from_str("(7Fxx,0030)").unwrap(),
            TagRange::from_str("(7FE0,13xx)").unwrap(),
            TagRange::from_str("(7FE0,0040)").unwrap(),
            TagRange::from_str("(7Fxx,0040)").unwrap(),
            TagRange::from_str("(7Fxx,0040)").unwrap(),
            TagRange::from_str("(7FE0,13xx)").unwrap(),
            TagRange::from_str("(7FE0,13xx)").unwrap(),
        ];
        let arr2 = [
            Tag::from_str("(7FE0,0030)").unwrap(),
            Tag::from_str("(7FE0,0030)").unwrap(),
            Tag::from_str("(7FE0,1354)").unwrap(),
            Tag::from_str("(7FE0,0030)").unwrap(),
            Tag::from_str("(7FE0,0050)").unwrap(),
            Tag::from_str("(7EE0,0040)").unwrap(),
            Tag::from_str("(7FE0,1454)").unwrap(),
            Tag::from_str("(7FF0,1354)").unwrap(),
        ];
        let arr3 = [true, true, true, false, false, false, false, false];
        let index = Vec::from_iter(0..arr1.len());
        assert_eq!(arr1.len(), arr2.len());
        assert_eq!(arr1.len(), arr3.len());
        for (a, (b, (c,i))) in arr1.iter().zip(arr2.iter().zip(arr3.iter().zip(index.iter()))) {
            assert_eq!(a.contains(b), *c, "failed on iteration: {}", i);
        }
    }

    #[test]
    fn tag_range_partial_eq() {
        let arr1 = [
            TagRange::from_str("(5372,1234)").unwrap(),
            TagRange::from_str("(53xx,1234)").unwrap(),
            TagRange::from_str("(53xx,1234)").unwrap(),
            TagRange::from_str("(5372,12xx)").unwrap(),
            TagRange::from_str("(5372,12xx)").unwrap(),
        ];
        let arr2 = [
            TagRange::from_str("(5372,1234)").unwrap(),
            TagRange::from_str("(53xx,1234)").unwrap(),
            TagRange::from_str("(5372,1234)").unwrap(),
            TagRange::from_str("(5372,12xx)").unwrap(),
            TagRange::from_str("(5372,1234)").unwrap(),
        ];
        let expected = [true, true, false, true, false];
        assert_eq!(arr1.len(), arr2.len());
        assert_eq!(arr1.len(), expected.len());
        for (a, (b, e)) in arr1.iter().zip(arr2.iter().zip(expected.iter())) {
            let c = a.eq(b);
            assert_eq!(*e, c);
        }
    }

    #[test]
    fn vm_from_str() {
        let arr1 = ["2", "n", "2n", "2-n", "2-2n", "2n-4n"];
        let arr2 = [
            VM {
                min: 2,
                max: 0,
                is_min_n: false,
                is_max_n: false,
            },
            VM {
                min: 0,
                max: 0,
                is_min_n: true,
                is_max_n: false,
            },
            VM {
                min: 2,
                max: 0,
                is_min_n: true,
                is_max_n: false,
            },
            VM {
                min: 2,
                max: 0,
                is_min_n: false,
                is_max_n: true,
            },
            VM {
                min: 2,
                max: 2,
                is_min_n: false,
                is_max_n: true,
            },
            VM {
                min: 2,
                max: 4,
                is_min_n: true,
                is_max_n: true,
            },
        ];
        let expected = [true, true, true, true, true, true];
        let index = Vec::from_iter(0..arr1.len());
        assert_eq!(arr1.len(), arr2.len());
        assert_eq!(arr1.len(), expected.len());
        for (a, (b, (e,i))) in arr1.iter().zip(arr2.iter().zip(expected.iter().zip(index.iter()))) {
            let c = VM::from_str(a);
            assert!(
                c.is_ok(),
                "failed at iteration [{}]: {}",
                i,
                c.err().unwrap()
            );
            let c = c.unwrap();
            assert_eq!(c == *b, *e);
        }
    }
}
