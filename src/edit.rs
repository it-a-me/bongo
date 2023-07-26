use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use lofty::{TagItem, TaggedFileExt};

pub fn edit(path: PathBuf, editor: Option<String>) -> Result<()> {
    let mut tagged = lofty::read_from_path(&path)?;
    let tags = tagged
        .tag_mut(tagged.primary_tag_type())
        .expect("untagged file");
    let mut map = HashMap::new();
    for tag in tags.items() {
        map.insert(format!("{:?}", tag.key()), format!("{:?}", tag.value()));
    }
    let toml = toml::to_string_pretty(&map)?;
    let tmp = tempfile::NamedTempFile::new()?.into_temp_path();
    std::fs::write(&tmp, toml)?;
    let editor = editor.unwrap_or(std::env::var("EDITOR").unwrap_or(String::from("vim")));
    let mut result: HashMap<String, String> = edit_toml(&tmp, &editor)?;
    for (key, value) in map {
        if let Some(val) = result.get(&key) {
            if val == &value {
                result.remove(&key);
            }
        }
    }
    for (key, value) in result {
        let err = format!(
            "failed to insert '{key}' => '{value}' in '{}'",
            path.to_string_lossy()
        );
        if !tags.insert(TagItem::new(
            lofty::ItemKey::Unknown(key),
            lofty::ItemValue::Text(value),
        )) {
            anyhow::bail!("{err}");
        }
    }

    Ok(())
}

fn edit_toml<T: serde::de::DeserializeOwned>(path: &Path, editor: &str) -> Result<T> {
    std::process::Command::new(editor).arg(path).status()?;
    match toml::from_str(&std::fs::read_to_string(path)?) {
        Ok(v) => Ok(v),
        Err(e) => {
            if dialoguer::Confirm::new()
                .with_prompt(&format!("{e}\n\nedit again?"))
                .interact()?
            {
                edit_toml(path, editor)
            } else {
                Err(e.into())
            }
        }
    }
}
