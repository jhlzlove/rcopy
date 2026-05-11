use anyhow::Result;
use arboard::Clipboard;
use std::io::{self, Write};

#[cfg(windows)]
use windows_sys::Win32::System::Console::SetConsoleOutputCP;

#[cfg(windows)]
const CP_UTF8: u32 = 65001;

fn main() -> Result<()> {
    // Windows 强制 UTF-8 输出
    #[cfg(windows)]
    unsafe {
        SetConsoleOutputCP(CP_UTF8);
    }

    let mut clipboard = Clipboard::new()?;
    let content = clipboard.get_text()?;

    io::stdout().write_all(content.as_bytes())?;

    Ok(())
}
