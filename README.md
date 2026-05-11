# clipboard-cli

A tiny cross-platform clipboard utility written in Rust.

`cpy` copies stdin into the system clipboard.  
`cpp` prints clipboard contents to stdout.

---

# Features

- Cross-platform clipboard support
- Unicode / emoji support
- Works in Git Bash on Windows
- Safe stdin handling (`read_to_end`)
- Handles non-UTF8 shell output gracefully
- Tiny and fast
- Text clipboard only

---

# Build

```bash
cargo build --release
```

---

# Usage

## Copy text

```bash
echo "hello world" | cpy
```

## Copy Unicode

```bash
echo "诗酒趁年华 😂" | cpy
```

## Paste clipboard

```bash
cpp
```

## Copy command output

```bash
ipconfig | cpy
```

```bash
systeminfo | cpy
```

```bash
wmic process list brief | cpy
```

---

# Notes

This tool currently supports text clipboard operations only.

Images, files, and rich content are not supported.

---

# Example

```bash
echo "Rust is awesome 🦀" | cpy

cpp
```

Output:

```text
Rust is awesome 🦀
```

---

# License

MIT
