use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context;

const CLIPBOARD_HOOK: &str = "PITHPACK_CLIPBOARD";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkKind {
    Stdout,
    File(PathBuf),
    Clipboard,
}

pub fn write(kind: &SinkKind, bundle: &str, stdout: &mut dyn Write) -> anyhow::Result<()> {
    match kind {
        SinkKind::Stdout => {
            stdout
                .write_all(bundle.as_bytes())
                .context("cannot write the Bundle to stdout")?;
            stdout.flush().context("cannot write the Bundle to stdout")
        }
        SinkKind::File(path) => write_atomically(path, bundle)
            .with_context(|| format!("cannot write the Bundle to `{}`", path.display())),
        SinkKind::Clipboard => match std::env::var(CLIPBOARD_HOOK) {
            Ok(hook) if hook.starts_with("file:") => {
                let path = Path::new(&hook["file:".len()..]);
                std::fs::write(path, bundle).with_context(|| {
                    format!("cannot write the clipboard hook file `{}`", path.display())
                })
            }
            _ => copy_to_clipboard(bundle).context("cannot copy the Bundle to the clipboard"),
        },
    }
}

fn write_atomically(path: &Path, bundle: &str) -> std::io::Result<()> {
    let dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    // Not `tempfile`: it creates 0600 files, which would leak into the renamed target.
    let tmp = dir.join(format!(".pithpack-{}.tmp", std::process::id()));
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .and_then(|mut f| f.write_all(bundle.as_bytes()))
        .and_then(|()| std::fs::rename(&tmp, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    written?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn copy_to_clipboard(bundle: &str) -> anyhow::Result<()> {
    arboard::Clipboard::new()?.set_text(bundle)?;
    Ok(())
}

// X11 and Wayland clipboards die with their owner, so a detached copy of this binary
// keeps ownership after the parent exits.
#[cfg(target_os = "linux")]
fn copy_to_clipboard(bundle: &str) -> anyhow::Result<()> {
    use std::process::{Command, Stdio};

    let mut child = Command::new(std::env::current_exe()?)
        .env(HOLD_ENV, "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .context("no stdin on the clipboard process")?
        .write_all(bundle.as_bytes())?;
    // The holder blocks for as long as it owns the clipboard, so only an early exit is a failure.
    for _ in 0..30 {
        if let Some(status) = child.try_wait()? {
            if status.success() {
                break;
            }
            let mut reason = String::new();
            if let Some(mut stderr) = child.stderr.take() {
                let _ = std::io::Read::read_to_string(&mut stderr, &mut reason);
            }
            anyhow::bail!("{}", reason.trim().trim_start_matches("error: "));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
const HOLD_ENV: &str = "PITHPACK_HOLD_CLIPBOARD";

/// Entry point of the detached clipboard owner; a no-op unless this process was spawned as one.
#[cfg(target_os = "linux")]
pub fn run_as_clipboard_holder() {
    use arboard::SetExtLinux;
    use std::io::Read;

    if std::env::var_os(HOLD_ENV).is_none() {
        return;
    }
    let mut text = String::new();
    let set = std::io::stdin()
        .read_to_string(&mut text)
        .map_err(anyhow::Error::from)
        .and_then(|_| Ok(arboard::Clipboard::new()?));
    match set {
        Ok(mut clipboard) => {
            if let Err(err) = clipboard.set().wait().text(text) {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
    std::process::exit(0);
}

#[cfg(not(target_os = "linux"))]
pub fn run_as_clipboard_holder() {}
