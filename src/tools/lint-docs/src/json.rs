use std::{collections::BTreeMap, error::Error, path::Path};

use serde::Serialize;

use crate::LintExtractor;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Level {
    Allow,
    Warn,
    Deny,
}

impl From<crate::Level> for Level {
    fn from(value: crate::Level) -> Self {
        match value {
            crate::Level::Allow => Level::Allow,
            crate::Level::Warn => Level::Warn,
            crate::Level::Deny => Level::Deny,
        }
    }
}

#[derive(Serialize)]
struct Span<'a> {
    path: &'a Path,
    line: usize,
}

#[derive(Serialize)]
struct Lint<'a> {
    id: &'a str,
    id_span: Span<'a>,
    docs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<&'a str>,
    level: Level,
}

impl<'a> LintExtractor<'a> {
    pub(crate) fn save_lints_json(
        &self,
        lints: &[crate::Lint],
        groups: &crate::groups::LintGroups,
    ) -> Result<(), Box<dyn Error>> {
        let lint_to_group = groups
            .iter()
            .flat_map(|(group, lints)| {
                lints.iter().map(move |lint| (lint.replace('-', "_"), group.as_str()))
            })
            .collect::<BTreeMap<_, _>>();

        let lints = lints
            .iter()
            .map(|lint| Lint {
                id: &lint.name,
                id_span: Span {
                    path: lint.path.strip_prefix(self.src_path.parent().unwrap()).unwrap(),
                    line: lint.lineno,
                },
                docs: lint.doc.join("\n"),
                group: lint_to_group.get(lint.name.as_str()).copied(),
                level: lint.level.into(),
            })
            .collect::<Vec<_>>();

        let serialized = serde_json::to_string_pretty(&lints)?;
        std::fs::create_dir_all(self.out_path)?;
        std::fs::write(self.out_path.join("lints.json"), serialized.as_bytes())?;
        Ok(())
    }
}
