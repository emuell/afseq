use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

/// enum for possible lua-ls built-in types
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LuaKind {
    Nil,
    Unknown,
    Any,
    Boolean,
    String,
    Number,
    Integer,
    Function,
    Table,
    Thread,
    UserData,
    Binary,
    LightUserData,
}

/// enum for complex custom types
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Unresolved(String),
    Lua(LuaKind),
    Array(Box<Kind>),
    Nullable(Box<Kind>),
    Table(Box<Kind>, Box<Kind>),
    Object(HashMap<String, Box<Kind>>),
    Alias(Box<Alias>),
    Class(Class),
    Function(Function),
    Enum(Vec<Kind>),
    EnumRef(Box<Enum>),
    SelfArg,
    Variadic(Box<Kind>),
    Literal(Box<LuaKind>, String),
}

/// a definition alias, rendered as a doc page
#[derive(Debug, Clone, PartialEq)]
pub struct Alias {
    pub file: Option<PathBuf>,
    pub line_number: Option<u32>,
    pub name: String,
    pub kind: Kind,
    pub desc: Option<String>,
}

/// variable definition used in fields and params and returns of functions
#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub file: Option<PathBuf>,
    pub line_number: Option<u32>,
    pub name: Option<String>,
    pub kind: Kind,
    pub desc: Option<String>,
    // pub default: String,
    // pub range: String
}

/// function definition for methods, functions and lambdas
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Function {
    pub file: Option<PathBuf>,
    pub line_number: Option<u32>,
    pub name: Option<String>,
    pub params: Vec<Var>,
    pub returns: Vec<Var>,
    pub desc: Option<String>,
    // pub overloads: ?
}

/// enumeration attached to classes
/// self.desc contains a code block string with the values
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub file: Option<PathBuf>,
    pub line_number: Option<u32>,
    pub name: String,
    pub desc: String,
}

/// scope of a class item
#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    Global,
    Local,
    Builtins,
    Modules,
}

/// class definition, rendered as a doc page
#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub file: Option<PathBuf>,
    pub line_number: Option<u32>,
    pub scope: Scope,
    pub name: String,
    pub fields: Vec<Var>,
    pub functions: Vec<Function>,
    pub enums: Vec<Enum>,
    pub constants: Vec<Var>,
    pub desc: String,
}

/// a wrapper for all top-level types coming from the json
#[derive(Debug, Clone, PartialEq)]
pub enum Def {
    Class(Class),
    Enum(Enum),
    Alias(Alias),
    Function(Function),
}

impl Kind {
    #[allow(unused)]
    pub fn collect_local_class_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();
        match self {
            Kind::Unresolved(_) => {}
            Kind::Lua(_lua_kind) => {}
            Kind::Array(item) => {
                types.extend(item.collect_local_class_types());
            }
            Kind::Nullable(item) => {
                types.extend(item.collect_local_class_types());
            }
            Kind::Table(key, value) => {
                types.extend(key.collect_local_class_types());
                types.extend(value.collect_local_class_types());
            }
            Kind::Object(map) => {
                for kind in map.values() {
                    types.extend(kind.collect_local_class_types());
                }
            }
            Kind::Alias(alias) => {}
            Kind::Class(class) => {
                if class.scope == Scope::Local {
                    types.insert(class.name.clone());
                }
                types.extend(class.collect_local_class_types());
            }
            Kind::Function(func) => {
                for ret in &func.returns {
                    types.extend(ret.kind.collect_local_class_types());
                }
                for param in &func.params {
                    types.extend(param.kind.collect_local_class_types());
                }
            }
            Kind::Enum(kinds) => {
                for kind in kinds {
                    types.extend(kind.collect_local_class_types());
                }
            }
            Kind::EnumRef(_) => {}
            Kind::SelfArg => {}
            Kind::Variadic(item) => {
                types.extend(item.collect_local_class_types());
            }
            Kind::Literal(_lua_kind, _) => {}
        }
        types
    }

    pub fn collect_alias_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();
        match self {
            Kind::Unresolved(_name) => {}
            Kind::Lua(_lua_kind) => {}
            Kind::Array(kind) => {
                types.extend(kind.collect_alias_types());
            }
            Kind::Nullable(item) => {
                types.extend(item.collect_alias_types());
            }
            Kind::Table(key, value) => {
                types.extend(key.collect_alias_types());
                types.extend(value.collect_alias_types());
            }
            Kind::Object(map) => {
                for kind in map.values() {
                    types.extend(kind.collect_alias_types());
                }
            }
            Kind::Alias(alias) => {
                types.insert(alias.name.clone());
            }
            Kind::Class(class) => {
                types.extend(class.collect_alias_types());
            }
            Kind::Function(function) => {
                for ret in &function.returns {
                    types.extend(ret.kind.collect_alias_types());
                }
                for param in &function.params {
                    types.extend(param.kind.collect_alias_types());
                }
            }
            Kind::Enum(kinds) => {
                for kind in kinds {
                    types.extend(kind.collect_alias_types());
                }
            }
            Kind::EnumRef(_enumref) => {}
            Kind::SelfArg => {}
            Kind::Variadic(item) => {
                types.extend(item.collect_alias_types());
            }
            Kind::Literal(_lua_kind, _) => {}
        }
        types
    }
}

impl Var {
    pub fn is_constant(&self) -> bool {
        self.name
            .as_ref()
            .is_some_and(|name| name.chars().all(|c| c.is_uppercase() || c == '_'))
    }

    pub fn is_not_constant(&self) -> bool {
        !self.is_constant()
    }
}

impl Function {
    pub fn strip_base(&self) -> Self {
        if let Some(name) = &self.name {
            Self {
                name: Some(
                    Class::get_end(name)
                        .map(|n| n.to_string())
                        .unwrap_or(self.name.clone().unwrap_or_default()),
                ),
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
}

impl Scope {
    pub fn from_name(name: &str) -> Self {
        if ["global", "pattern"].contains(&name) {
            Scope::Global
        } else if ["bit", "debug", "io", "math", "os", "table"].contains(&name) {
            Scope::Modules
        } else {
            Scope::Local
        }
    }

    pub fn path_prefix(&self) -> String {
        match self {
            Scope::Global | Scope::Local => String::from("API/"),
            Scope::Builtins => String::from("API/builtins/"),
            Scope::Modules => String::from("API/modules/"),
        }
    }
}

impl Class {
    pub fn get_base(s: &str) -> Option<&str> {
        s.rfind('.').map(|pos| &s[..pos])
    }

    pub fn get_end(s: &str) -> Option<&str> {
        s.rfind('.').map(|pos| &s[pos + 1..])
    }

    pub fn collect_local_aliases(&self, aliases: &HashMap<String, Alias>) -> HashSet<String> {
        let mut local_alias_names = self.collect_alias_types();

        // loop until recursion settled
        loop {
            let mut new_local_alias_names = local_alias_names.clone();

            // find aliases names in aliases
            for name in new_local_alias_names.clone() {
                let alias = aliases.get(&name).unwrap();
                new_local_alias_names.extend(alias.kind.collect_alias_types());
            }

            // resolve new aliases
            for alias in new_local_alias_names.clone().into_iter() {
                if let Some(alias) = aliases.get(&alias) {
                    if let Kind::Alias(alias) = &alias.kind {
                        new_local_alias_names.insert(alias.name.clone());
                    }
                }
            }

            if new_local_alias_names != local_alias_names {
                local_alias_names.clone_from(&new_local_alias_names);
            } else {
                break;
            }
        }

        local_alias_names
    }

    #[allow(unused)]
    pub fn collect_local_class_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();
        for field in &self.fields {
            types.extend(field.kind.collect_local_class_types());
        }
        for function in &self.functions {
            for ret in &function.returns {
                types.extend(ret.kind.collect_local_class_types());
            }
            for param in &function.params {
                types.extend(param.kind.collect_local_class_types());
            }
        }
        for con in &self.constants {
            types.extend(con.kind.collect_local_class_types());
        }
        types
    }

    pub fn collect_alias_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();
        for field in &self.fields {
            types.extend(field.kind.collect_alias_types());
        }
        for function in &self.functions {
            for ret in &function.returns {
                types.extend(ret.kind.collect_alias_types());
            }
            for param in &function.params {
                types.extend(param.kind.collect_alias_types());
            }
        }
        for con in &self.constants {
            types.extend(con.kind.collect_alias_types());
        }
        types
    }
}

// ---------------------------------------- debug helpers to show types

impl LuaKind {
    pub fn show(&self) -> String {
        let s = serde_json::to_string(self).unwrap();
        s.trim_matches('"').to_string()
    }
}

impl Kind {
    pub fn has_unresolved(&self) -> bool {
        let s = format!("{}", self);
        s.contains("\x1b[33m")
    }
}

impl Var {
    pub fn has_unresolved(&self) -> bool {
        self.kind.has_unresolved()
    }
    pub fn show(&self) -> String {
        format!(
            "Var {} : {}",
            self.name.clone().unwrap_or_default(),
            self.kind
        )
    }
}

impl Enum {
    pub fn show(&self) -> String {
        format!("Enum {}", self.name)
    }
}

impl Function {
    pub fn has_unresolved(&self) -> bool {
        for p in &self.params {
            if p.has_unresolved() {
                return true;
            }
        }
        for r in &self.returns {
            if r.has_unresolved() {
                return true;
            }
        }
        false
    }
}

impl Alias {
    pub fn show(&self) -> String {
        format!("Alias {} {}", self.name, self.kind)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved(_) => write!(f, "\x1b[33m{:?}\x1b[0m", self),
            Self::Nullable(b) => write!(f, "Nullable({})", b.as_ref()),
            Self::Array(b) => write!(f, "Array({})", b.as_ref()),
            Self::Table(k, v) => write!(f, "Table({}, {})", k.as_ref(), v.as_ref()),
            Self::Enum(ks) => write!(
                f,
                "Enum({})",
                ks.iter()
                    .map(|k| format!("{}", k))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Object(hm) => {
                write!(
                    f,
                    "Object({})",
                    hm.iter()
                        .map(|(k, v)| format!("{} : {}", k, v))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Self::Function(fun) => write!(f, "{}", fun),
            Self::Variadic(v) => write!(f, "Variadic({})", v.as_ref()),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name.clone() {
            write!(f, "{} : {}", name, self.kind)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| format!("{}", p))
            .collect::<Vec<String>>()
            .join(", ");
        let returns = self
            .returns
            .iter()
            .map(|r| format!("{}", r))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "Function {}({}){}",
            self.name.clone().unwrap_or_default(),
            params,
            if returns.is_empty() {
                String::default()
            } else {
                format!(" -> {}", returns)
            }
        )
    }
}

impl Class {
    pub fn has_unresolved(&self) -> bool {
        for v in &self.fields {
            if v.has_unresolved() {
                return true;
            }
        }
        for f in &self.functions {
            if f.has_unresolved() {
                return true;
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.enums.is_empty() && self.functions.is_empty()
    }

    fn with_new_line(s: &str) -> String {
        if s.is_empty() {
            s.to_string()
        } else {
            format!("\n{}", s)
        }
    }

    pub fn show(&self) -> String {
        format!(
            "  Class {}{}{}{}",
            self.name,
            Self::with_new_line(
                &self
                    .enums
                    .iter()
                    .map(|e| format!("    {}", Enum::show(e)))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Self::with_new_line(
                &self
                    .fields
                    .iter()
                    .map(|v| format!("    {}", Var::show(v)))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Self::with_new_line(
                &self
                    .functions
                    .iter()
                    .map(|f| format!("    {}", f))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        )
    }
}

// impl Def {
//     pub fn show(&self) -> String {
//         match self {
//             Self::Alias(d) => d.show(),
//             Self::Function(d) => format!("{}", d),
//             Self::Class(d) => d.show(),
//             Self::Enum(e) => e.show(),
//         }
//     }
// }
