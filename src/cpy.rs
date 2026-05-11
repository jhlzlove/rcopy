use anyhow::Result;
use arboard::Clipboard;
use encoding_rs::GBK;
use std::io::{self, Read};

fn main() -> Result<()> {
    let mut buffer = Vec::new();

    io::stdin().read_to_end(&mut buffer)?;

    // 优先尝试 UTF-8
    let text = match String::from_utf8(buffer.clone()) {
        Ok(s) => s,

        // Windows 常见 fallback：GBK
        Err(_) => {
            let (cow, _, _) = GBK.decode(&buffer);
            cow.into_owned()
        }
    };

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;

    Ok(())
}
