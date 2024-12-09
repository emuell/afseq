use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use itertools::Itertools;

use crate::{library::Library, types::*};

impl Library {
    /// render each page inside the library as a list of string tuples (name, content)
    pub fn export_docs(&self) -> Vec<(String, String)> {
        // split classes into globals and modules
        let mut globals = vec![];
        for (path, classes) in self.classes_by_file_in_scopes(&[Scope::Global, Scope::Local]) {
            if path.to_string_lossy().is_empty() {
                // skip classes which have no file path (Lua internals)
                continue;
            }
            let file_stem = path
                .file_stem()
                .map(|v| v.to_string_lossy())
                .expect("expecting class to have a valid source file path");
            let mut content = String::new();
            content.push_str("\n<!-- toc -->\n\n");

            for class in Self::sort_classes(classes) {
                let url_root = "../";
                content.push_str(&class.render(url_root, &self.aliases));
                content.push_str("\n\n");
            }
            globals.push((file_stem.to_string(), content));
        }

        let mut modules = vec![];
        for class in self.classes_in_scopes(&[Scope::Modules]) {
            let url_root = "../../";
            let content = class.render(url_root, &self.aliases);
            modules.push((String::from("modules/") + &class.name, content));
        }

        // add builtin classes
        let mut builtins = vec![];
        for class in Library::builtin_classes() {
            let url_root = "../../";
            let content = class.render(url_root, &self.aliases);
            builtins.push((String::from("builtins/") + &class.name, content));
        }

        // create final docs
        let mut docs: Vec<(String, String)> = vec![];
        docs.append(&mut globals);

        if !modules.is_empty() {
            docs.push(("modules".to_string(), "# Lua Module Extensions".to_string()));
            docs.append(&mut modules);
        }
        if !builtins.is_empty() {
            docs.push(("builtins".to_string(), "# Lua Builtin Types".to_string()));
            docs.append(&mut builtins);
        }
        docs = docs
            .iter()
            .unique_by(|(name, _)| name.to_ascii_lowercase())
            .cloned()
            .collect::<Vec<_>>();
        Self::sort_docs(docs)
    }

    fn classes_in_scopes(&self, scopes: &[Scope]) -> Vec<Class> {
        self.classes
            .values()
            .filter(|&c| scopes.contains(&c.scope))
            .cloned()
            .collect()
    }

    fn classes_by_file_in_scopes(&self, scopes: &[Scope]) -> HashMap<PathBuf, Vec<Class>> {
        let mut map = HashMap::<PathBuf, Vec<Class>>::new();
        for class in self.classes_in_scopes(scopes) {
            let file = class.file.clone().unwrap_or_default();
            if let Some(classes) = map.get_mut(&file) {
                classes.push(class.clone());
            } else {
                map.insert(file.clone(), vec![class.clone()]);
            }
        }
        map
    }

    fn sort_classes(mut classes: Vec<Class>) -> Vec<Class> {
        let custom_weight = |name: &str| -> usize {
            if name == "global" {
                0
            } else if name.ends_with("Context") {
                99
            } else {
                1
            }
        };
        classes.sort_by_key(|class| (custom_weight(&class.name), class.name.to_lowercase()));
        classes
    }

    fn sort_docs(mut docs: Vec<(String, String)>) -> Vec<(String, String)> {
        let custom_weight = |name: &str| -> usize {
            if name == "global" {
                0
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

fn file_link(text: &str, url: &str) -> String {
    format!("[`{}`]({}.md)", text, url)
}

fn class_link(text: &str, url: &str, hash: &str) -> String {
    format!("[`{}`]({}.md#{})", text, url, hash)
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
        file_link(&text, &(format!("{}API/builtins/", url_root) + &text))
    }
}

impl Kind {
    fn link(&self, url_root: &str, file: &Path) -> String {
        match self {
            Kind::Lua(lk) => lk.link(url_root),
            Kind::Literal(k, s) => match k.as_ref() {
                LuaKind::String => format!("`\"{}\"`", s),
                LuaKind::Integer | LuaKind::Number => format!("`{}`", s.clone()),
                _ => s.clone(),
            },
            Kind::Class(class) => {
                if matches!(class.scope, Scope::Global | Scope::Local) {
                    let file = class.file.clone().unwrap_or(PathBuf::new());
                    let file_stem = file
                        .file_stem()
                        .map(|v| v.to_string_lossy())
                        .unwrap_or("[unknown file]".into());
                    class_link(
                        &class.name,
                        &(url_root.to_string() + &class.scope.path_prefix() + &file_stem),
                        &class.name,
                    )
                } else {
                    file_link(
                        &class.name,
                        &(url_root.to_string() + &class.scope.path_prefix() + &class.name),
                    )
                }
            }
            Kind::Enum(kinds) => kinds
                .iter()
                .map(|k| k.link(url_root, file))
                .collect::<Vec<String>>()
                .join(" | "),
            Kind::EnumRef(enumref) => {
                let file = enumref.file.clone().unwrap_or(PathBuf::new());
                let file_stem = file
                    .file_stem()
                    .map(|v| v.to_string_lossy())
                    .unwrap_or("[unknown file]".into());
                enum_link(
                    &enumref.name,
                    &(url_root.to_string() + &Scope::Global.path_prefix() + &file_stem),
                    &enumref.name,
                )
            }
            Kind::SelfArg => format!("[*self*]({}API/builtins/self.md)", url_root),
            Kind::Array(k) => format!("{}[]", k.link(url_root, file)),
            Kind::Nullable(k) => format!(
                "{}{}",
                k.as_ref().link(url_root, file),
                file_link("?", &format!("{}API/builtins/nil", url_root))
            ),
            Kind::Alias(alias) => alias_link(&alias.name, &alias.name),
            Kind::Function(f) => f.short(url_root, file),
            Kind::Table(k, v) => format!(
                "table<{}, {}>",
                k.as_ref().link(url_root, file),
                v.as_ref().link(url_root, file)
            ),
            Kind::Object(hm) => {
                let mut keys = hm.iter().map(|(k, _)| k.clone()).collect::<Vec<String>>();
                keys.sort();
                let fields = keys
                    .iter()
                    .map(|k| format!("{} : {}", k, hm.get(k).unwrap().link(url_root, file)))
                    .collect::<Vec<String>>()
                    .join(", "); // TODO print on newlines?
                format!("{{ {} }}", fields)
            }
            Kind::Variadic(k) => format!("...{}", k.link(url_root, file)),
            Kind::Unresolved(s) => s.clone(),
        }
    }
}

impl Var {
    fn short(&self, url_root: &str, file: &Path) -> String {
        if matches!(self.kind, Kind::SelfArg) {
            self.kind.link(url_root, file)
        } else if let Some(name) = self.name.clone() {
            format!("{} : {}", name, self.kind.link(url_root, file))
        } else {
            self.kind.link(url_root, file)
        }
    }
    fn long(&self, url_root: &str, file: &Path) -> String {
        let desc = self.desc.clone().unwrap_or_default();
        format!(
            "{}{}",
            hash(
                &h3(&self.short(url_root, file)),
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
    fn render(&self, url_root: &str, file: &Path) -> String {
        format!(
            "{}\n{}  \n{}",
            hash(&h3(&self.name), &self.name),
            self.kind.link(url_root, file),
            self.desc
                .clone()
                .map(|d| description(d.as_str()))
                .unwrap_or_default()
        )
    }
}

impl Function {
    fn long(&self, url_root: &str, file: &Path) -> String {
        let name = self.name.clone().unwrap_or("fun".to_string());
        if self.params.is_empty() {
            let name = hash(&h3(&format!("`{}()`", &name)), &name);
            self.with_desc(&self.with_returns(&name, url_root, file))
        } else {
            let params = self
                .params
                .iter()
                .map(|v| v.short(url_root, file))
                .collect::<Vec<String>>()
                .join(", ");

            self.with_desc(&self.with_returns(
                &hash(&format!("### {}({})", &name, params), &name),
                url_root,
                file,
            ))
        }
    }
    fn short(&self, url_root: &str, file: &Path) -> String {
        if self.params.is_empty() && self.returns.is_empty() {
            return self.empty();
        }
        let returns = Self::render_vars(&self.returns, url_root, file);
        format!(
            "{}({}){}",
            &self.name.clone().unwrap_or_default(),
            Self::render_vars(&self.params, url_root, file),
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
    fn render_vars(vars: &[Var], url_root: &str, file: &Path) -> String {
        vars.iter()
            .map(|v| v.short(url_root, file))
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
    fn with_returns(&self, head: &str, url_root: &str, file: &Path) -> String {
        let returns = self
            .returns
            .iter()
            .map(|v| v.short(url_root, file))
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
        let name = if self.name == "global" {
            "Global"
        } else {
            &self.name
        };
        let mut content = vec![h1(&hash(name, name))];

        if !self.desc.is_empty() {
            content.push(description(&self.desc))
        }

        if !self.enums.is_empty() || !self.constants.is_empty() {
            let enums = &self.enums;
            let constants = &self.constants;
            content.push(format!(
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
                    .map(|v| v.long(url_root, &self.file.clone().unwrap_or_default()))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        if !self.fields.is_empty() {
            content.push("\n---".to_string());
            content.push(format!(
                "{}\n{}\n",
                h2("Properties"),
                self.fields
                    .iter()
                    .map(|v| v.long(url_root, &self.file.clone().unwrap_or_default()))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        let functions = &self.functions;
        if !functions.is_empty() {
            content.push("\n---".to_string());
            content.push(format!(
                "{}\n{}",
                h2("Functions"),
                functions
                    .iter()
                    .map(|f| f.long(url_root, &self.file.clone().unwrap_or_default()))
                    .collect::<Vec<String>>()
                    .join("\n")
            ))
        }

        // append used local aliases
        let local_alias_names = self.collect_local_aliases(aliases);
        if !local_alias_names.is_empty() {
            content.push("\n\n\n---".to_string());
            content.push(h2("Aliases"));
            let mut alias_names: Vec<&String> = aliases.keys().collect();
            alias_names.sort();
            for name in alias_names {
                if local_alias_names.contains(name) {
                    content.push(
                        aliases
                            .get(name)
                            .unwrap()
                            .render(url_root, &self.file.clone().unwrap_or_default()),
                    );
                    content.push(String::new());
                }
            }
        }

        content.push("\n".to_string());
        content.join("  \n")
    }
}
