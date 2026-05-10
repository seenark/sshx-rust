use crate::error::SshxError;

pub fn copy_to_clipboard(text: &str, no_clipboard: bool) -> Result<(), SshxError> {
    if no_clipboard {
        println!("{text}");
        return Ok(());
    }

    match arboard::Clipboard::new() {
        Ok(mut cb) => cb.set_text(text).map_err(|e| SshxError::ClipboardUnavailable {
            reason: e.to_string(),
        })?,
        Err(e) => {
            eprintln!("Warning: clipboard unavailable ({}), printing instead:", e);
            println!("{text}");
        }
    }
    Ok(())
}