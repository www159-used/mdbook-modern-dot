use mdbook_preprocessor::errors::Result;
use std::io;
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

const DOT_MISSING: &str = "Graphviz `dot` was not found on PATH. Install Graphviz:\n  \
    macOS:   brew install graphviz\n  \
    Linux:   sudo apt install graphviz  (or your distro's graphviz package)\n  \
    Windows: https://graphviz.org/download/";
const DOT_FAILED: &str = "Graphviz `dot` command failed (check diagram syntax or run `dot -V`)";

pub fn ensure_available() -> Result<()> {
    std::process::Command::new("dot")
        .arg("-V")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(map_spawn_error)?;
    Ok(())
}

pub async fn render_to_svg(arguments: &[String], source: &str) -> Result<String> {
    let output = run_dot(arguments, source).await?.wait_with_output().await?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(dot_failed_error())
    }
}

pub async fn render_to_file(arguments: &[String], source: &str, output_path: &Path) -> Result<()> {
    let output_path = output_path
        .to_str()
        .ok_or_else(invalid_output_path)?
        .to_string();

    let mut args = arguments.to_vec();
    args.extend(["-o".into(), output_path]);

    if run_dot(&args, source).await?.wait().await?.success() {
        Ok(())
    } else {
        Err(dot_failed_error())
    }
}

async fn run_dot(arguments: &[String], source: &str) -> Result<Child> {
    let mut child = Command::new("dot")
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(map_spawn_error)?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(source.as_bytes()).await?;
    }

    Ok(child)
}

fn map_spawn_error(error: io::Error) -> mdbook_preprocessor::errors::Error {
    if error.kind() == io::ErrorKind::NotFound {
        io::Error::new(io::ErrorKind::NotFound, DOT_MISSING).into()
    } else {
        error.into()
    }
}

fn dot_failed_error() -> mdbook_preprocessor::errors::Error {
    io::Error::new(io::ErrorKind::InvalidData, DOT_FAILED).into()
}

fn invalid_output_path() -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, "invalid diagram output path")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_install_hint() {
        let error = map_spawn_error(io::Error::new(
            io::ErrorKind::NotFound,
            "No such file or directory",
        ));
        assert!(error.to_string().contains("Graphviz `dot` was not found"));
        assert!(error.to_string().contains("brew install graphviz"));
    }
}

