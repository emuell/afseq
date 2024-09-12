use std::collections::HashMap;
use std::vec;

use crate::{library::Library, types::*};

use itertools::Itertools;

impl Library {
    /// render each page inside the library as a list of string tuples (name, content)
    pub fn export_docs(&self) -> Vec<(String, String)> {
        // split classes int globals, builtins and structs
        let mut globals = vec![];
        let mut builtins = vec![];
        let mut modules = vec![];
        for (name, class) in &self.classes {
            // url_root is the path to /API folder
            let url_root = if matches!(class.scope, Scope::Modules | Scope::Builtins) {
                "../../"
            } else {
                "../"
            };
            let content = class.render(url_root, &self.aliases);
            match class.scope {
                Scope::Global | Scope::Local => globals.push((name.clone(), content)),
                Scope::Builtins => builtins.push((String::from("builtins/") + name, content)),
                Scope::Modules => modules.push((String::from("modules/") + name, content)),
            }
        }
        let mut docs: Vec<(String, String)> = vec![];
        docs.append(&mut globals);
        if !modules.is_empty() {
            docs.push(("modules".to_string(), "#Lua Standard Modules".to_string()));
            docs.append(&mut modules);
        }
        if !builtins.is_empty() {
            docs.push(("builtins".to_string(), "#Lua Builtin Types".to_string()));
            docs.append(&mut builtins);
        }
        docs = docs
            .iter()
            .unique_by(|(name, _)| name.to_ascii_lowercase())
            .cloned()
            .collect::<Vec<_>>();
        Self::sort_docs(docs)
    }

    fn sort_docs(mut docs: Vec<(String, String)>) -> Vec<(String, String)> {
        let custom_weight = |name: &str| -> usize {
            if name == "global" {
                0
            } else if name.ends_with("Context") {
                98
            } else if name.starts_with("modules") {
                99
            } else if name.starts_with("builtins") {
                100
            } else {
                10
            }
        };
        docs.sort_by_key(|(name, _)| (custom_weight(name), name.to_lowercase()));
        docs
    }
}

fn heading(text: &str, level: usize) -> String {
    format!("{} {}", "#".repeat(level), text)
}

fn h1(text: &str) -> String {
    heading(text, 1)
}

fn h2(text: &str) -> String {
    heading(text, 2)
}

fn h3(text: &str) -> String {
    heading(text, 3)
}

fn link(text: &str, url: &str) -> String {
    format!("[`{}`]({}.md)", text, url)
}
fn enum_link(text: &str, url: &str, hash: &str) -> String {
    format!("[`{}`]({}.md#{})", text, url, hash)
}
fn alias_link(text: &str, hash: &str) -> String {
    format!("[`{}`](#{})", text, hash)
}

fn quote(text: &str) -> String {
    format!("> {}", text.replace('\n', "\n> "))
}

fn description(desc: &str) -> String {
    quote(
        desc.replace("### examples", "#### examples")
            .trim_matches('\n'),
    )
}

// fn item(text: &str) -> String {
//     format!("* {}", text)
// }
// fn italic(text: &str) -> String {
//     format!("*{}*", text)
// }

fn hash(text: &str, hash: &str) -> String {
    format!("{} {{#{}}}", text, hash)
}

impl LuaKind {
    fn link(&self, url_root: &str) -> String {
        let text = self.show();
        link(&text, &(format!("{}API/builtins/", url_root) + &text))
    }
}

impl Kind {
    fn link(&self, url_root: &str) -> String {
        match self {
            Kind::Lua(lk) => lk.link(url_root),
            Kind::Literal(k, s) => match k.as_ref() {
                LuaKind::String => format!("`\"{}\"`", s),
                LuaKind::Integer | LuaKind::Number => format!("`{}`", s.clone()),
                _ => s.clone(),
            },
            Kind::Class(scope, name) => {
                link(name, &(url_root.to_string() + &scope.path_prefix() + name))
            }
            Kind::Enum(kinds) => kinds
                .iter()
                .map(|k| k.link(url_root))
                .collect::<Vec<String>>()
                .join(" | "),
            Kind::EnumRef(name) => enum_link(
                name,
                Class::get_base(name).unwrap_or(name),
                Class::get_end(name).unwrap_or_default(),
            ),
            Kind::SelfArg => format!("[*self*]({}API/builtins/self.md)", url_root),
            Kind::Array(k) => format!("{}[]", k.link(url_root)),
            Kind::Nullable(k) => format!(
                "{}{}",
                k.as_ref().link(url_root),
                link("?", &format!("{}API/builtins/nil", url_root))
            ),
            Kind::Alias(name) => alias_link(name, name),
            Kind::Function(f) => f.short(url_root),
            Kind::Table(k, v) => format!(
                "table<{}, {}>",
                k.as_ref().link(url_root),
                v.as_ref().link(url_root)
            ),
            Kind::Object(hm) => {
                let mut keys = hm.iter().map(|(k, _)| k.clone()).collect::<Vec<String>>();
                keys.sort();
                let fields = keys
                    .iter()
                    .map(|k| format!("{} : {}", k, hm.get(k).unwrap().link(url_root)))
                    .collect::<Vec<String>>()
                    .join(", "); // TODO print on newlines?
                format!("{{ {} }}", fields)
            }
            Kind::Variadic(k) => format!("...{}", k.link(url_root)),
            Kind::Unresolved(s) => s.clone(),
        }
    }
}

impl Var {
    fn short(&self, url_root: &str) -> String {
        if matches!(self.kind, Kind::SelfArg) {
            self.kind.link(url_root)
        } else if let Some(name) = self.name.clone() {
            format!("{} : {}", name, self.kind.link(url_root))
        } else {
            self.kind.link(url_root)
        }
    }
    fn long(&self, url_root: &str) -> String {
        let desc = self.desc.clone().unwrap_or_default();
        format!(
            "{}{}",
            hash(
                &h3(&self.short(url_root)),
                &self.name.clone().unwrap_or_default()
            ),
            if desc.is_empty() {
                desc
            } else {
                format!("\n{}\n", description(&desc))
            }
        )
    }
}

impl Alias {
    fn render(&self, url_root: &str) -> String {
        format!(
            "{}\n{}  \n{}",
            hash(&h3(&self.name), &self.name),
            self.kind.link(url_root),
            self.desc
                .clone()
                .map(|d| description(d.as_str()))
                .unwrap_or_default()
        )
    }
}

impl Function {
    fn long(&self, url_root: &str) -> String {
        let name = self.name.clone().unwrap_or("fun".to_string());
        if self.params.is_empty() {
            let name = hash(&h3(&format!("`{}()`", &name)), &name);
            self.with_desc(&self.with_returns(&name, url_root))
        } else {
            let params = self
                .params
                .iter()
                .map(|v| v.short(url_root))
                .collect::<Vec<String>>()
                .join(", ");

            self.with_desc(&self.with_returns(
                &hash(&format!("### {}({})", &name, params), &name),
                url_root,
            ))
        }
    }
    fn short(&self, url_root: &str) -> String {
        if self.params.is_empty() && self.returns.is_empty() {
            return self.empty();
        }
        let returns = Self::render_vars(&self.returns, url_root);
        format!(
            "{}({}){}",
            &self.name.clone().unwrap_or_default(),
            Self::render_vars(&self.params, url_root),
            if returns.is_empty() {
                returns
            } else {
                format!(" `->` {}", returns)
            }
        )
    }
    fn empty(&self) -> String {
        format!("{}()", &self.name.clone().unwrap_or("fun".to_string()))
    }
    fn render_vars(vars: &[Var], url_root: &str) -> String {
        vars.iter()
            .map(|v| v.short(url_root))
            .collect::<Vec<String>>()
            .join(", ")
    }
    fn with_desc(&self, head: &str) -> String {
        let desc = self.desc.clone().unwrap_or_default();
        if desc.is_empty() {
            head.to_string()
        } else {
            format!("{}\n{}", head, description(&desc))
        }
    }
    fn with_returns(&self, head: &str, url_root: &str) -> String {
        let returns = self
            .returns
            .iter()
            .map(|v| v.short(url_root))
            .collect::<Vec<String>>()
            .join(", ");
        if returns.is_empty() {
            head.to_string()
        } else {
            format!("{}\n`->`{}  \n", head, returns)
        }
    }
}

impl Class {
    fn render(&self, url_root: &str, aliases: &HashMap<String, Alias>) -> String {
        let mut m = vec![h1(&self.name)];

        if !self.desc.is_empty() {
            m.push(description(&self.desc))
        }

        m.push("\n<!-- toc -->\n".to_string());

        if !self.enums.is_empty() || !self.constants.is_empty() {
            let enums = &self.enums;
            let constants = &self.constants;
            m.push(format!(
                "{}\n{}\n{}",
                h2("Constants"),
                enums
                    .iter()
                    .map(|e| {
                        let name = e.name.clone();
                        let end = Class::get_end(&name).unwrap_or(&name);
                        format!("{}\n{}", hash(&h3(end), end), description(&e.desc))
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
                constants
                    .iter()
                    .map(|v| v.long(url_root))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        if !self.fields.is_empty() {
            m.push("\n---".to_string());
            m.push(format!(
                "{}\n{}\n",
                h2("Properties"),
                self.fields
                    .iter()
                    .map(|v| v.long(url_root))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        let functions = &self.functions;
        if !functions.is_empty() {
            m.push("\n---".to_string());
            m.push(format!(
                "{}\n{}",
                h2("Functions"),
                functions
                    .iter()
                    .map(|f| f.long(url_root))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        // append used local aliases
        if self.scope != Scope::Local {
            let local_alias_names = self.collect_local_aliases(aliases);

            // append all used local aliases
            if !local_alias_names.is_empty() {
                m.push("\n\n\n---".to_string());
                m.push(h2("Aliases"));
                let mut alias_names: Vec<&String> = aliases.keys().collect();
                alias_names.sort();
                for name in alias_names {
                    if local_alias_names.contains(name) {
                        m.push(aliases.get(name).unwrap().render(url_root));
                        m.push(String::new());
                    }
                }
            }
        }

        m.push("\n".to_string());
        m.join("  \n")
    }
}
