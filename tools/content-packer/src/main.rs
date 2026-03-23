use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use schema::{compile_author_root, parse_author_root};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let input = PathBuf::from(args.next().context("missing input path")?);
    let output = PathBuf::from(args.next().context("missing output path")?);
    let input_text = fs::read_to_string(&input)
        .with_context(|| format!("failed to read {}", input.display()))?;
    let author = parse_author_root(&input_text)
        .with_context(|| format!("failed to parse {}", input.display()))?;
    let compiled = compile_author_root(author);
    let bytes = compiled.encode().context("failed to encode compiled content")?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&output, bytes).with_context(|| format!("failed to write {}", output.display()))?;
    println!("packed content -> {}", output.display());
    Ok(())
}

