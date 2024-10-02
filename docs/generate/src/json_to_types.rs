use std::path::PathBuf;

use crate::{
    json::{ArgDef, ArgType, Definition, Doc, Extend, Field, ReturnDef, Type},
    lua_parser::LuaParser,
    types::*,
};

impl Def {
    pub fn from_definition(d: &Definition) -> Option<Self> {
        if let Some(first) = d.defines.first() {
            match first.lua_type {
                Type::Doc(Doc::Class) => Some(Self::Class(Class::from_definition(
                    d,
                    &first.file,
                    first.start,
                ))),

                // TODO parse fields in enum either from desc or from Type::TableField
                Type::Doc(Doc::Enum) => Some(Self::Enum(Enum {
                    file: Some(first.file.clone().into()),
                    line_number: Some(first.start),
                    name: d.name.clone(),
                    desc: d.rawdesc.clone().unwrap_or_default(),
                })),

                Type::Doc(Doc::Alias) => Some(Self::Alias(Alias {
                    file: Some(first.file.clone().into()),
                    line_number: Some(first.start),
                    desc: d.rawdesc.clone(),
                    name: d.name.clone(),
                    kind: first
                        .clone()
                        .extends
                        .map(|e| Kind::from_string(&e.view))
                        .unwrap_or(Kind::Unresolved(d.name.clone())),
                })),

                Type::SetField | Type::SetGlobal => {
                    if let Some(extend) = first.extends.clone() {
                        match extend.lua_type {
                            Type::Lua(LuaKind::Function) => Function::from_extend(
                                extend,
                                first.file.clone().into(),
                                d.name.clone(),
                                d.rawdesc.clone().unwrap_or_default(),
                            )
                            .map(Self::Function),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Kind {
    fn from_string(s: &str) -> Self {
        LuaParser::type_def(s)
    }
}

impl From<&String> for Kind {
    fn from(s: &String) -> Self {
        LuaParser::type_def(s.as_str())
    }
}
impl From<Type> for Kind {
    fn from(t: Type) -> Self {
        match t {
            Type::Lua(lk) => Self::Lua(lk),
            _ => Self::Unresolved(format!("{:?}", t)),
        }
    }
}
impl From<Extend> for Kind {
    fn from(e: Extend) -> Self {
        // Self::Unresolved("missing type".to_string())
        if let Some(types) = e.types {
            if types.len() > 1 {
                Self::Enum(types.iter().map(|t| Self::from_string(&t.view)).collect())
            } else if let Some(first) = types.first() {
                if first.view.clone() + "?" == e.view {
                    Self::from_string(&e.view)
                } else {
                    Self::from_string(&first.view)
                }
            } else {
                Self::from_string(&e.view)
            }
        } else {
            Self::from(e.lua_type)
        }
    }
}

impl Var {
    fn from_field(field: Field) -> Option<Self> {
        Some(Self {
            file: Some(field.file.clone().into()),
            line_number: Some(field.start),
            kind: field
                .extends
                .map(Kind::from)
                .unwrap_or(Kind::Unresolved(format!(
                    "there are no extends on field {}",
                    field.name.clone()
                ))),
            name: Some(field.name),
            desc: field.rawdesc,
        })
    }
    fn from_argdef(ad: &ArgDef) -> Option<Self> {
        Some(Self {
            file: None,
            line_number: None,
            kind: match ad.lua_type {
                ArgType::SelfArg => Kind::SelfArg,
                ArgType::Local => Kind::from_string(&ad.view),
                ArgType::Variadic => Kind::Variadic(Box::new(Kind::from_string(&ad.view))),
            },
            name: ad.name.clone(),
            desc: None, // desc: ad.desc.unwrap_or_default(),
        })
    }

    fn from_return(rd: &ReturnDef) -> Option<Self> {
        Some(Self {
            file: None,
            line_number: None,
            kind: Kind::from_string(&rd.view),
            // kind: match rd.lua_type {
            //     ArgType::SelfArg => ArgKind::SelfKind,
            //     ArgType::Local => ArgKind::from_string(rd.view.as_str()),
            //     ArgType::Variadic => ArgKind::Variadic,
            // },
            name: rd.name.clone(),
            desc: None, // desc: rd.desc.unwrap_or_default(),
        })
    }
}

impl Function {
    fn from_extend(extend: Extend, file: PathBuf, name: String, desc: String) -> Option<Self> {
        match extend.lua_type {
            Type::Lua(LuaKind::Function) => {
                let params = extend
                    .args
                    .iter()
                    .filter_map(Var::from_argdef)
                    .collect::<Vec<_>>();
                let returns = extend
                    .returns
                    .iter()
                    .filter_map(Var::from_return)
                    .collect::<Vec<Var>>();
                Some(Self {
                    file: Some(file),
                    line_number: Some(extend.start),
                    name: Some(name),
                    params,
                    returns,
                    desc: Some(desc.to_string()).filter(|s| !s.is_empty()),
                })
            }
            _ => None,
        }
    }
    fn from_field(field: Field) -> Option<Self> {
        if let Some(extend) = field.extends {
            Self::from_extend(
                extend,
                field.file.into(),
                field.name.clone(),
                field.rawdesc.unwrap_or_default(),
            )
        } else {
            None
        }
    }
}

impl Class {
    fn from_definition(d: &Definition, file: &str, line_number: u32) -> Self {
        Self {
            file: Some(file.into()),
            line_number: Some(line_number),
            scope: Scope::from_name(&d.name),
            name: d.name.clone(),
            fields: d
                .fields
                .clone()
                .into_iter()
                .filter(Field::is_field)
                .filter_map(Var::from_field)
                .collect(),
            functions: d
                .fields
                .clone()
                .into_iter()
                .filter(Field::is_function)
                .filter_map(Function::from_field)
                .collect(),
            enums: vec![], // enums will get added in Library
            constants: vec![],
            desc: d.rawdesc.clone().unwrap_or_default(),
        }
    }
}
