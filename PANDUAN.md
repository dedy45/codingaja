# Panduan Instalasi & Penggunaan Claw Code

> **Lokasi instalasi:** `C:\Users\dedy\Documents\ClawCode`

---

## 1. Prasyarat

Pastikan sudah terinstall:

| Tool | Versi Minimum | Status |
|------|--------------|--------|
| **Git** | 2.x | ✅ `git --version` |
| **Python** | 3.10+ | ✅ `python --version` |
| **Rust** | 1.70+ (via rustup) | ✅ `rustc --version` |
| **Cargo** | (bundled dengan Rust) | ✅ `cargo --version` |

---

## 2. Struktur Repository

```
ClawCode/
├── src/                    # Python porting workspace
│   ├── main.py             # CLI entrypoint
│   ├── commands.py         # Command definitions
│   ├── tools.py            # Tool definitions
│   ├── runtime.py          # Session runtime
│   └── ...                 # 60+ modul lainnya
├── rust/                   # Rust port (utama)
│   ├── Cargo.toml          # Workspace manifest
│   └── crates/
│       ├── api/            # API client + streaming
│       ├── runtime/        # Session, tools, MCP, config
│       ├── claw-cli/       # Binary utama (REPL)
│       ├── plugins/        # Plugin system
│       ├── commands/       # Slash commands
│       ├── server/         # HTTP/SSE server
│       ├── lsp/            # LSP client
│       └── tools/          # Tool specs
├── tests/                  # Python unit tests
└── assets/                 # Gambar & referensi
```

---

## 3. Instalasi Langkah demi Langkah

### Langkah 1: Clone Repository

```bash
cd C:\Users\dedy\Documents
git clone https://github.com/instructkr/claw-code.git ClawCode
cd ClawCode
```

### Langkah 2: Verifikasi Python

Jalankan test suite untuk memastikan Python workspace berfungsi:

```bash
python -m unittest discover -s tests -v
```

**Output yang diharapkan:** 22 tests passed, OK

### Langkah 3: Build Rust Binary

```bash
cd rust
cargo build --release
```

**Waktu build:** ~2-3 menit (tergantung hardware)
**Output binary:** `rust\target\release\claw.exe` (~10 MB)

---

## 4. Cara Menggunakan

### 4.1 Python CLI (Manifest & Summary)

```bash
# Ringkasan porting workspace
python -m src.main summary

# Manifest file Python
python -m src.main manifest

# List subsystem packages
python -m src.main subsystems --limit 16

# List commands
python -m src.main commands --limit 10

# List tools
python -m src.main tools --limit 10

# Parity audit
python -m src.main parity-audit

# Bootstrap session
python -m src.main bootstrap "review MCP tool" --limit 5

# Command graph
python -m src.main command-graph

# Tool pool
python -m src.main tool-pool
```

### 4.2 Rust CLI (`claw.exe`) — **Yang Utama**

Tambahkan ke PATH atau gunakan path absolut:

```powershell
# Tambahkan ke PATH (PowerShell)
$env:PATH += ";C:\Users\dedy\Documents\ClawCode\rust\target\release"

# Atau buat alias permanen
New-Alias claw "C:\Users\dedy\Documents\ClawCode\rust\target\release\claw.exe"
```

#### Mode Interaktif (REPL)

```bash
claw
```

Memulai sesi interaktif dengan slash commands (`/help`, `/status`, `/compact`, dll).

#### Mode Prompt Langsung

```bash
# Non-interactive prompt
claw "jelaskan fungsi main.rs"

# Dengan model spesifik
claw --model opus "summarize this repo"

# Output JSON
claw --output-format json prompt "explain src/main.rs"

# Batasi tools
claw --allowedTools read,glob "summarize Cargo.toml"
```

#### Manajemen Sesi

```bash
# Resume sesi tersimpan
claw --resume session.json /status /diff /export notes.txt

# List agents
claw agents

# List skills
claw /skills

# Login/logout
claw login
claw logout
```

#### Slash Commands (di dalam REPL)

| Command | Deskripsi |
|---------|-----------|
| `/help` | Tampilkan semua command |
| `/status` | Status sesi saat ini |
| `/compact` | Kompak history sesi |
| `/model [model]` | Ganti model AI |
| `/permissions [mode]` | Set permission mode |
| `/clear` | Mulai sesi baru |
| `/cost` | Token usage |
| `/config [section]` | Inspeksi config |
| `/memory` | Instruction memory |
| `/bughunter [scope]` | Cari bug di codebase |
| `/commit` | Generate commit message |
| `/commit-push-pr` | Commit, push, buka PR |
| `/pr [context]` | Draft pull request |
| `/issue [context]` | Draft GitHub issue |
| `/ultraplan [task]` | Deep planning |
| `/teleport <path>` | Jump ke file/symbol |
| `/plugin [action]` | Manage plugins |
| `/agents` | List agents |
| `/skills` | List skills |
| `/export [file]` | Export conversation |
| `/session [action]` | Manage sessions |

---

## 5. Setup PATH Permanen (Opsional)

Agar `claw` bisa dipanggil dari mana saja:

### PowerShell (Current User)

```powershell
# Tambahkan ke PATH user
$clawPath = "C:\Users\dedy\Documents\ClawCode\rust\target\release"
$oldPath = [Environment]::GetEnvironmentVariable("Path", "User")
[Environment]::SetEnvironmentVariable("Path", "$oldPath;$clawPath", "User")

# Restart terminal, lalu test
claw --help
```

### Atau Buat Shortcut

Buat file `claw.cmd` di folder yang sudah ada di PATH:

```cmd
@echo off
"C:\Users\dedy\Documents\ClawCode\rust\target\release\claw.exe" %*
```

---

## 6. Troubleshooting

| Masalah | Solusi |
|---------|--------|
| `python -m src.main` error | Pastikan di folder `ClawCode/` root |
| `cargo build` gagal | Pastikan Rust terinstall: `rustup update` |
| `claw.exe` tidak ditemukan | Gunakan path absolut atau tambahkan ke PATH |
| Test Python gagal | Pastikan Python 3.10+: `python --version` |

---

## 7. Verifikasi Instalasi

Jalankan semua command ini untuk memastikan semuanya berfungsi:

```bash
# 1. Python tests
python -m unittest discover -s tests -v

# 2. Python CLI
python -m src.main summary

# 3. Rust binary
C:\Users\dedy\Documents\ClawCode\rust\target\release\claw.exe --help

# 4. Rust binary (prompt mode)
C:\Users\dedy\Documents\ClawCode\rust\target\release\claw.exe "hello world"
```

---

## 8. Referensi

- **Repository:** https://github.com/instructkr/claw-code
- **Komunitas:** https://instruct.kr/ (Discord)
- **Sponsor:** https://github.com/sponsors/instructkr
