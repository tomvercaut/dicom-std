use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Lifetime {
    pub name: String,
}

impl Default for Lifetime {
    fn default() -> Self {
        Self {
            name: "".to_string(),
        }
    }
}

impl Lifetime {
    /// Create a new lifetime with a name.
    ///
    /// # Arguments
    /// * `name` - label of the lifetime
    pub fn new(name: &str) -> Result<Self> {
        let n = name.len();
        if n == 0 {
            return Err(Error::LifetimeNew("an empty name is not valid".to_string()));
        }
        let t: &str;
        if let Some(p) = name.find("'") {
            t = &name[p + 1..];
        } else {
            t = name;
        }

        let r = t.find(|c: char| !c.is_ascii_alphanumeric());
        if r.is_some() {
            return Err(Error::LifetimeNew(format!(
                "Lifetime name is not valid: {}",
                name
            )));
        }
        Ok(Self {
            name: t.to_string(),
        })
    }

    /// Check if the lifetime has no value.
    ///
    /// Returns true if the lifetime has no name, false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    /// Check if the lifetime has value.
    ///
    /// Returns true if the lifetime has a name, false otherwise.
    #[inline]
    pub fn has_value(&self) -> bool {
        !self.is_empty()
    }
}

impl FromStr for Lifetime {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let t = s.trim();
        return if !t.starts_with("'") {
            Err(Error::LifetimeParseError(s.to_string()))
        } else {
            let u = &t[1..];
            if !u.chars().all(char::is_alphabetic) {
                Err(Error::LifetimeParseError(s.to_string()))
            } else {
                Ok(Self {
                    name: (&t[1..]).to_string(),
                })
            }
        };
    }
}

impl Display for Lifetime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "'{}", self.name.as_str())
    }
}

impl Eq for Lifetime {}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FnArg {
    pub reference: bool,
    pub mutable: bool,
    pub lifetime: Option<Lifetime>,
    pub name: String,
    pub object: String,
}

impl Default for FnArg {
    fn default() -> Self {
        Self {
            reference: false,
            mutable: false,
            lifetime: None,
            name: "".to_string(),
            object: "".to_string(),
        }
    }
}

impl Display for FnArg {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: ", &self.name)?;
        if self.reference {
            write!(f, "&")?;
            if self.lifetime.is_some() {
                let lf = self.lifetime.as_ref().unwrap();
                if lf.has_value() {
                    write!(f, "{} ", lf)?;
                }
            }
            if self.mutable {
                write!(f, "mut ")?
            }
        }
        write!(f, "{}", &self.object)
    }
}

impl Eq for FnArg {}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReturnValue {
    pub reference: bool,
    pub mutable: bool,
    lifetime: Option<Lifetime>,
    object: String,
}

impl ReturnValue {
    pub(crate) fn is_empty(&self) -> bool {
        self.object.is_empty()
    }
}

impl Default for ReturnValue {
    fn default() -> Self {
        Self {
            reference: false,
            mutable: false,
            lifetime: None,
            object: "".to_string(),
        }
    }
}

impl Display for ReturnValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.reference {
            write!(f, "&")?;

            if self.lifetime.is_some() {
                write!(f, "{}", self.lifetime.as_ref().unwrap())?;
            }

            if self.mutable {
                write!(f, " mut ")?;
            } else {
                write!(f, " ")?;
            }
        }
        if !self.object.is_empty() {
            write!(f, "{}", self.object.trim())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Visibility {
    Private,
    Public,
    PublicCrate,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Visibility::Private => Ok(()),
            Visibility::Public => {
                write!(f, "pub")
            }
            Visibility::PublicCrate => {
                write!(f, "pub(crate)")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StructField {
    pub visibility: Visibility,
    pub name: String,
    pub lifetime: Option<Lifetime>,
    pub reference: bool,
    pub type_: String,
    pub depth: i16,
}

impl Default for StructField {
    fn default() -> Self {
        Self {
            visibility: Default::default(),
            name: "".to_string(),
            lifetime: None,
            reference: false,
            type_: "".to_string(),
            depth: 0,
        }
    }
}

impl Display for StructField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.visibility != Visibility::Private {
            write!(f, "{} ", self.visibility)?;
        }
        write!(f, "{}: ", self.name.as_str())?;
        if self.reference {
            write!(f, "&")?;
            if self.lifetime.is_some() {
                write!(f, "{} ", self.lifetime.as_ref().unwrap())?;
            }
        }
        write!(f, "{}", self.type_.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Struct {
    /// Types that this struct is derived from
    pub derives: Vec<String>,
    /// Visibility of the struct
    pub visibility: Visibility,
    /// Name of the struct [type]
    pub name: String,
    /// Lifetimes linked with this struct
    pub lifetimes: Vec<Lifetime>,
    /// Field members of the struct
    pub fields: Vec<StructField>,
}

impl Default for Struct {
    fn default() -> Self {
        Self {
            derives: vec![],
            visibility: Default::default(),
            name: "".to_string(),
            lifetimes: vec![],
            fields: vec![],
        }
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if !self.is_valid() {
            return Err(std::fmt::Error);
        }
        if !self.derives.is_empty() {
            write!(f, "#[derive(")?;
            let mut first = true;
            for d in &self.derives {
                if !first {
                    write!(f, ", {}", d.trim())?;
                } else {
                    write!(f, "{}", d.trim())?;
                    first = false;
                }
            }
            write!(f, ")]\n")?;
        }
        if self.visibility != Visibility::Private {
            write!(f, "{} ", &self.visibility)?;
        }
        write!(f, "struct {}", self.name.as_str())?;
        let mut nlf: usize = 0;
        for lf in &self.lifetimes {
            if lf.has_value() {
                if nlf == 0 {
                    write!(f, "<")?;
                    nlf += 1;
                } else {
                    write!(f, ", ")?;
                }
                write!(f, "{}", lf)?;
            }
        }
        if nlf > 0 {
            write!(f, ">")?;
        }
        write!(f, " {{\n")?;
        for field in &self.fields {
            write!(f, "    {},\n", field)?;
        }
        write!(f, "}}\n")?;
        Ok(())
    }
}

impl Struct {
    pub fn is_valid(&self) -> bool {
        for d in &self.derives {
            if d.trim().is_empty() {
                return false;
            }
        }
        if self.name.is_empty() {
            return false;
        }
        for c in self.name.chars() {
            if !c.is_ascii_alphanumeric() {
                return false;
            }
        }
        let mut b = true;
        for lf in &self.lifetimes {
            if lf.is_empty() {
                b = false;
                break;
            }
        }
        if !b {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TraitFunction {
    pub name: String,
    pub member: bool,
    pub mutable: bool,
    pub lifetimes: Vec<Lifetime>,
    pub args: Vec<FnArg>,
    pub rvalue: ReturnValue,
}

impl Default for TraitFunction {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            member: true,
            mutable: false,
            lifetimes: vec![],
            args: vec![],
            rvalue: Default::default(),
        }
    }
}

impl Display for TraitFunction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if !self.is_valid() {
            return Err(std::fmt::Error);
        }
        write!(f, "fn {}", self.name)?;
        let mut first = true;
        for lf in &self.lifetimes {
            if lf.has_value() {
                if first {
                    write!(f, "<{}", lf)?;
                    first = false;
                } else {
                    write!(f, ", {}", lf)?;
                }
            }
        }
        if !first {
            write!(f, ">")?;
        }
        write!(f, "(")?;
        if self.member {
            if self.mutable {
                write!(f, "&mut self")?;
            } else {
                write!(f, "&self")?;
            }
        }
        first = true;
        for arg in &self.args {
            if first {
                if self.member {
                    write!(f, ", ")?;
                }
                write!(f, "{}", arg)?;
                first = false;
            } else {
                write!(f, ", {}", arg)?;
            }
        }
        write!(f, ") ")?;
        if !self.rvalue.is_empty() {
            write!(f, "-> {}", self.rvalue)?;
        }
        write!(f, ";")
    }
}

impl TraitFunction {
    pub fn is_valid(&self) -> bool {
        if self.name.is_empty() {
            return false;
        }
        for c in self.name.chars() {
            if !c.is_ascii_alphanumeric() && c != '_' {
                return false;
            }
        }
        let mut b = true;
        for lf in &self.lifetimes {
            if lf.is_empty() {
                b = false;
                break;
            }
        }
        if !b {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Trait {
    pub visibility: Visibility,
    pub name: String,
    pub derives: Vec<Box<Trait>>,
    pub lifetimes: Vec<Lifetime>,
    pub functions: Vec<TraitFunction>,
}

impl Default for Trait {
    fn default() -> Self {
        Self {
            visibility: Default::default(),
            name: "".to_string(),
            derives: vec![],
            lifetimes: vec![],
            functions: vec![],
        }
    }
}

impl Display for Trait {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if !self.is_valid() {
            return Err(std::fmt::Error);
        }
        if self.visibility != Visibility::Private {
            write!(f, "{} ", &self.visibility)?;
        }
        write!(f, "trait {}", self.name.as_str())?;
        let mut b = true;
        for lf in &self.lifetimes {
            if b {
                write!(f, "<{}", lf)?;
                b = false;
            } else {
                write!(f, "{}", lf)?;
            }
        }
        if !b {
            write!(f, ">")?;
        }
        if !self.derives.is_empty() {
            write!(f, ": ")?;
            b = true;
            for dv in &self.derives {
                if b {
                    write!(f, "{}", dv.name)?;
                    b = false;
                } else {
                    write!(f, " + {}", dv.name)?;
                }
            }
        }
        write!(f, " ")?;
        write!(f, "{{\n")?;
        for tf in &self.functions {
            write!(f, "    {}\n", tf)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl Trait {
    pub fn is_valid(&self) -> bool {
        if self.name.is_empty() {
            return false;
        }
        for c in self.name.chars() {
            if !c.is_ascii_alphanumeric() && c == '_' {
                return false;
            }
        }
        let mut b = true;
        for lf in &self.lifetimes {
            if lf.is_empty() {
                b = false;
                break;
            }
        }
        if !b {
            return false;
        }
        for tr in &self.derives {
            if !tr.is_valid() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use log::LevelFilter;

    use crate::rust::syntax::{FnArg, Lifetime, Struct, StructField, TraitFunction, Visibility};
    use crate::rust::{ReturnValue, Trait};
    use crate::Error;

    #[allow(dead_code)]
    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn lifetime_from_str() {
        let arr = ["'a", "a", "'a1", "'a1_"];
        let exp = [
            Ok(Lifetime {
                name: "a".to_string(),
            }),
            Err(Error::LifetimeParseError("".to_string())),
            Err(Error::LifetimeParseError("".to_string())),
            Err(Error::LifetimeParseError("".to_string())),
        ];
        assert_eq!(arr.len(), exp.len());
        for (a, e) in arr.iter().zip(exp.iter()) {
            let t = Lifetime::from_str(*a);
            match t {
                Ok(lifetime) => match e {
                    Ok(e_lifetime) => {
                        assert_eq!(&lifetime, e_lifetime);
                    }
                    Err(err) => {
                        assert!(
                            false,
                            "obtained a Lifetime instance but expected an error: {}",
                            err
                        );
                    }
                },
                Err(_) => match e {
                    Ok(e_lifetime) => {
                        assert!(
                            false,
                            "failed to obtain a Lifetime instance but expected: {}",
                            e_lifetime
                        );
                    }
                    Err(err) => match err {
                        Error::LifetimeParseError(_) => {
                            assert!(true);
                        }
                        _ => {
                            assert!(false, "expected a LifetimeParseError to be generated");
                        }
                    },
                },
            }
        }
    }

    #[test]
    fn lifetime_display() {
        let lf = Lifetime::from_str("'a");
        assert!(lf.is_ok(), "{}", lf.err().unwrap());
        let lf = lf.unwrap();
        let s = format!("{}", &lf);
        assert_eq!(s.as_str(), "'a");
    }

    #[test]
    fn lifetime_to_string() {
        let lf = Lifetime::from_str("'a");
        assert!(lf.is_ok(), "{}", lf.err().unwrap());
        let lf = lf.unwrap();
        let s = lf.to_string();
        assert_eq!(s.as_str(), "'a");
    }

    #[test]
    fn fnarg_display() {
        let arr = [
            FnArg {
                reference: false,
                mutable: false,
                lifetime: Default::default(),
                name: "arg".to_string(),
                object: "Obj".to_string(),
            },
            FnArg {
                reference: true,
                mutable: false,
                lifetime: Default::default(),
                name: "arg".to_string(),
                object: "Obj".to_string(),
            },
            FnArg {
                reference: true,
                mutable: true,
                lifetime: Default::default(),
                name: "arg".to_string(),
                object: "Obj".to_string(),
            },
            FnArg {
                reference: true,
                mutable: true,
                lifetime: Lifetime::new("a").unwrap().into(),
                name: "arg".to_string(),
                object: "Obj".to_string(),
            },
        ];
        let e_arr = ["arg: Obj", "arg: &Obj", "arg: &mut Obj", "arg: &'a mut Obj"];
        for (a, e) in arr.iter().zip(e_arr.iter()) {
            let t = format!("{}", a);
            assert_eq!(t.as_str(), *e);
        }
    }

    #[test]
    fn struct_field_display() {
        let arr = [
            StructField {
                visibility: Default::default(),
                name: "value".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
            },
            StructField {
                visibility: Visibility::Public,
                name: "value".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
            },
            StructField {
                visibility: Visibility::PublicCrate,
                name: "value".to_string(),
                lifetime: None,
                reference: false,
                type_: "Obj".to_string(),
                depth: 0,
            },
            StructField {
                visibility: Default::default(),
                name: "value".to_string(),
                lifetime: None,
                reference: true,
                type_: "Obj".to_string(),
                depth: 0,
            },
            StructField {
                visibility: Default::default(),
                name: "value".to_string(),
                lifetime: Lifetime::new("a").unwrap().into(),
                reference: true,
                type_: "Obj".to_string(),
                depth: 0,
            },
        ];
        let e_arr = [
            "value: Obj",
            "pub value: Obj",
            "pub(crate) value: Obj",
            "value: &Obj",
            "value: &'a Obj",
        ];
        assert_eq!(arr.len(), e_arr.len());
        for (a, e) in arr.iter().zip(e_arr.iter()) {
            let t = format!("{}", a);
            assert_eq!(t.as_str(), *e);
        }
    }

    #[test]
    fn struct_display() {
        let arr = [Struct {
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            visibility: Visibility::Public,
            name: "Obj".to_string(),
            lifetimes: vec![Lifetime::new("a").unwrap(), Lifetime::new("b").unwrap()],
            fields: vec![
                StructField {
                    visibility: Visibility::Public,
                    name: "name".to_string(),
                    lifetime: Lifetime::new("a").unwrap().into(),
                    reference: true,
                    type_: "str".to_string(),
                    depth: 0,
                },
                StructField {
                    visibility: Visibility::PublicCrate,
                    name: "data".to_string(),
                    lifetime: Lifetime::new("b").unwrap().into(),
                    reference: true,
                    type_: "str".to_string(),
                    depth: 0,
                },
            ],
        }];
        let e_arr = [r#"#[derive(Debug, Clone)]
pub struct Obj<'a, 'b> {
    pub name: &'a str,
    pub(crate) data: &'b str,
}
"#];
        assert_eq!(arr.len(), e_arr.len());
        for (a, e) in arr.iter().zip(e_arr.iter()) {
            let t = format!("{}", a);
            assert_eq!(t.as_str(), *e);
        }
    }

    #[test]
    fn trait_function_display() {
        let arr = [
            TraitFunction {
                name: "get_data".to_string(),
                member: false,
                mutable: false,
                lifetimes: vec![],
                args: vec![],
                rvalue: ReturnValue {
                    reference: false,
                    mutable: false,
                    lifetime: None,
                    object: "String".to_string(),
                },
            },
            TraitFunction {
                name: "get_data_str".to_string(),
                member: true,
                mutable: true,
                lifetimes: vec![Lifetime::new("a").unwrap()],
                args: vec![FnArg {
                    reference: true,
                    mutable: false,
                    lifetime: Some(Lifetime::new("a").unwrap()),
                    name: "s".to_string(),
                    object: "str".to_string(),
                }],
                rvalue: ReturnValue {
                    reference: false,
                    mutable: false,
                    lifetime: None,
                    object: "String".to_string(),
                },
            },
        ];
        let e_arr = [
            "fn get_data() -> String;",
            "fn get_data_str<'a>(&mut self, s: &'a str) -> String;",
        ];
        assert_eq!(arr.len(), e_arr.len());
        for (a, e) in arr.iter().zip(e_arr.iter()) {
            let t = format!("{}", a);
            assert_eq!(t.as_str(), *e);
        }
    }

    #[test]
    fn trait_display() {
        let arr = [Trait {
            visibility: Visibility::Public,
            name: "Trait".to_string(),
            derives: vec![],
            lifetimes: vec![Lifetime::new("a").unwrap()],
            functions: vec![
                TraitFunction {
                    name: "set".to_string(),
                    member: true,
                    mutable: true,
                    lifetimes: vec![],
                    args: vec![FnArg {
                        reference: true,
                        mutable: false,
                        lifetime: None,
                        name: "data".into(),
                        object: "str".to_string(),
                    }],
                    rvalue: ReturnValue {
                        reference: false,
                        mutable: false,
                        lifetime: None,
                        object: "bool".to_string(),
                    },
                },
                TraitFunction {
                    name: "get".to_string(),
                    member: true,
                    mutable: false,
                    lifetimes: vec![Lifetime::new("a").unwrap()],
                    args: vec![],
                    rvalue: ReturnValue {
                        reference: true,
                        mutable: false,
                        lifetime: Some(Lifetime::new("a").unwrap()),
                        object: "str".to_string(),
                    },
                },
            ],
        }];
        let e_arr = [r#"pub trait Trait<'a> {
    fn set(&mut self, data: &str) -> bool;
    fn get<'a>(&self) -> &'a str;
}"#];

        assert_eq!(arr.len(), e_arr.len());
        for (a, e) in arr.iter().zip(e_arr.iter()) {
            let t = format!("{}", a);
            assert_eq!(t.as_str(), *e);
        }
    }
}
