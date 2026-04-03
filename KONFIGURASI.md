# Panduan Lengkap Konfigurasi Claw Code

## 📂 Lokasi File Konfigurasi

Claw membaca konfigurasi dari **5 lokasi** (urutan prioritas rendah → tinggi):

| # | File | Scope | Keterangan |
|---|------|-------|------------|
| 1 | `%USERPROFILE%\.claw.json` | Global (user) | Legacy, semua project |
| 2 | `%USERPROFILE%\.claw\settings.json` | Global (user) | Config utama user |
| 3 | `<project>/.claw.json` | Project | Config per project |
| 4 | `<project>/.claw/settings.json` | Project | Config project alternatif |
| 5 | `<project>/.claw/settings.local.json` | Local | Override lokal (gitignored) |

> **Prioritas:** File di bawah menimpa file di atas. Config project menimpa config global.

---

## 🗂️ Struktur Folder

```
C:\Users\dedy\
├── .claw\                          # Folder config global
│   ├── settings.json               # ← Config utama user (EDIT INI)
│   ├── credentials.json            # ← OAuth tokens (auto-generated)
│   ├── plugins\                    # ← Plugin terinstall
│   └── sessions\                   # ← Session tersimpan
│
└── Documents\ClawCode\             # Project folder
    ├── .claw.json                  # ← Config project (EDIT INI)
    └── .claw\
        ├── settings.json           # ← Config project alternatif
        └── settings.local.json     # ← Override lokal (gitignored)
```

---

## 🔧 File Konfigurasi Utama

### 1. Config Global User — `%USERPROFILE%\.claw\settings.json`

Config ini berlaku untuk **semua project**.

```powershell
# Buat folder jika belum ada
mkdir $env:USERPROFILE\.claw

# Buat file config
notepad $env:USERPROFILE\.claw\settings.json
```

### 2. Config Project — `<project>/.claw.json`

Config ini hanya berlaku untuk **project ini**.

```powershell
# Sudah ada di C:\Users\dedy\Documents\ClawCode\.claw.json
notepad C:\Users\dedy\Documents\ClawCode\.claw.json
```

---

## 📝 Template Konfigurasi Lengkap

### Template Minimal

```json
{
  "model": "gpt-4o",
  "permissions": {
    "defaultMode": "workspace-write"
  }
}
```

### Template Lengkap (Semua Opsi)

```json
{
  "model": "gpt-4o",
  "permissions": {
    "defaultMode": "workspace-write"
  },
  "hooks": {
    "PreToolUse": ["scripts/pre-tool.sh"],
    "PostToolUse": ["scripts/post-tool.sh"]
  },
  "plugins": {
    "enabled": {
      "memory": true,
      "github": false
    },
    "externalDirectories": ["./my-plugins"],
    "installRoot": "%USERPROFILE%/.claw/plugins"
  },
  "mcp": {
    "servers": {
      "filesystem": {
        "type": "stdio",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
      },
      "web-search": {
        "type": "sse",
        "url": "http://localhost:3000/sse"
      }
    }
  },
  "sandbox": {
    "enabled": false,
    "filesystemMode": "workspace-only"
  }
}
```

---

## ⚙️ Penjelasan Setiap Opsi

### `model` — Model AI Default

Model yang dipakai kalau tidak pakai `--model`.

| Value | Provider | API Key |
|-------|----------|---------|
| `"opus"` | Anthropic/Claw | `ANTHROPIC_API_KEY` |
| `"sonnet"` | Anthropic/Claw | `ANTHROPIC_API_KEY` |
| `"haiku"` | Anthropic/Claw | `ANTHROPIC_API_KEY` |
| `"claude-opus-4-6"` | Anthropic/Claw | `ANTHROPIC_API_KEY` |
| `"claude-sonnet-4-6"` | Anthropic/Claw | `ANTHROPIC_API_KEY` |
| `"grok"` / `"grok-3"` | xAI | `XAI_API_KEY` |
| `"grok-mini"` / `"grok-3-mini"` | xAI | `XAI_API_KEY` |
| `"gpt-4o"` | OpenAI | `OPENAI_API_KEY` |
| `"o1"` | OpenAI | `OPENAI_API_KEY` |
| Nama model custom | OpenAI-compatible | `OPENAI_API_KEY` + `OPENAI_BASE_URL` |

### `permissions.defaultMode` — Mode Izin

| Value | Deskripsi |
|-------|-----------|
| `"read-only"` | Hanya baca, tidak bisa edit file |
| `"workspace-write"` | Bisa edit file di workspace |
| `"danger-full-access"` | Full akses (hati-hati!) |
| `"default"` | Sama dengan `read-only` |
| `"auto"` | Sama dengan `workspace-write` |
| `"dontAsk"` | Sama dengan `danger-full-access` |

### `hooks` — Hook Script

Jalankan script sebelum/sesudah tool digunakan.

```json
{
  "hooks": {
    "PreToolUse": ["scripts/check-before.sh"],
    "PostToolUse": ["scripts/log-after.sh"]
  }
}
```

### `plugins` — Plugin

```json
{
  "plugins": {
    "enabled": {
      "memory": true,
      "github": true,
      "docker": false
    },
    "externalDirectories": ["./plugins", "../shared-plugins"],
    "installRoot": "%USERPROFILE%/.claw/plugins",
    "registryPath": "https://plugins.claw.dev/registry.json"
  }
}
```

### `mcp.servers` — MCP Server

Tambahkan MCP server untuk tool tambahan.

```json
{
  "mcp": {
    "servers": {
      "filesystem": {
        "type": "stdio",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
      },
      "github": {
        "type": "stdio",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-github"],
        "env": {
          "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxx"
        }
      },
      "web-search": {
        "type": "sse",
        "url": "http://localhost:3000/sse"
      },
      "custom-http": {
        "type": "http",
        "url": "http://localhost:8080/mcp"
      }
    }
  }
}
```

### `sandbox` — Isolasi Keamanan

```json
{
  "sandbox": {
    "enabled": false,
    "filesystemMode": "workspace-only",
    "namespaceRestrictions": true,
    "networkIsolation": false,
    "allowedMounts": ["/tmp", "./data"]
  }
}
```

| Opsi | Value | Deskripsi |
|------|-------|-----------|
| `filesystemMode` | `"off"` | Tidak ada isolasi |
| | `"workspace-only"` | Hanya akses folder project |
| | `"allow-list"` | Hanya folder yang di-mount |
| `networkIsolation` | `true/false` | Blokir akses network |
| `allowedMounts` | `["/path"]` | Folder yang boleh diakses |

### `oauth` — OAuth Config (jarang perlu diedit)

```json
{
  "oauth": {
    "clientId": "claw-code-client",
    "authorizeUrl": "https://auth.instruct.kr/authorize",
    "tokenUrl": "https://auth.instruct.kr/token",
    "callbackPort": 4545,
    "scopes": ["user:inference", "user:sessions:claw_code"]
  }
}
```

---

## 🚀 Contoh Konfigurasi Siap Pakai

### 1. Pakai OpenAI (GPT-4o)

**File:** `%USERPROFILE%\.claw\settings.json`

```json
{
  "model": "gpt-4o",
  "permissions": {
    "defaultMode": "workspace-write"
  }
}
```

Lalu set API key:
```powershell
[System.Environment]::SetEnvironmentVariable("OPENAI_API_KEY", "sk-xxx", "User")
```

### 2. Pakai xAI (Grok-3)

**File:** `%USERPROFILE%\.claw\settings.json`

```json
{
  "model": "grok-3",
  "permissions": {
    "defaultMode": "workspace-write"
  }
}
```

Lalu set API key:
```powershell
[System.Environment]::SetEnvironmentVariable("XAI_API_KEY", "xai-xxx", "User")
```

### 3. Pakai Ollama (Lokal, Gratis)

**File:** `%USERPROFILE%\.claw\settings.json`

```json
{
  "model": "llama3",
  "permissions": {
    "defaultMode": "workspace-write"
  }
}
```

Lalu set env var:
```powershell
[System.Environment]::SetEnvironmentVariable("OPENAI_API_KEY", "ollama", "User")
[System.Environment]::SetEnvironmentVariable("OPENAI_BASE_URL", "http://localhost:11434/v1", "User")
```

### 4. Pakai OpenRouter

**File:** `%USERPROFILE%\.claw\settings.json`

```json
{
  "model": "google/gemini-2.0-flash",
  "permissions": {
    "defaultMode": "workspace-write"
  }
}
```

Lalu set API key:
```powershell
[System.Environment]::SetEnvironmentVariable("OPENAI_API_KEY", "sk-or-xxx", "User")
[System.Environment]::SetEnvironmentVariable("OPENAI_BASE_URL", "https://openrouter.ai/api/v1", "User")
```

### 5. Pakai LM Studio (Lokal)

**File:** `%USERPROFILE%\.claw\settings.json`

```json
{
  "model": "local-model",
  "permissions": {
    "defaultMode": "workspace-write"
  }
}
```

Lalu set env var:
```powershell
[System.Environment]::SetEnvironmentVariable("OPENAI_API_KEY", "lm-studio", "User")
[System.Environment]::SetEnvironmentVariable("OPENAI_BASE_URL", "http://localhost:1234/v1", "User")
```

---

## 🔍 Cek Config yang Aktif

```powershell
# Cek model yang aktif
claw /config model

# Cek semua config
claw /config

# Cek hooks
claw /config hooks
```

---

## ⚡ Quick Reference

| Mau apa? | Edit file | Set env var |
|----------|-----------|-------------|
| Ganti model | `model` di `.claw.json` | - |
| Pakai OpenAI | - | `OPENAI_API_KEY` |
| Pakai Ollama | - | `OPENAI_API_KEY` + `OPENAI_BASE_URL` |
| Pakai xAI/Grok | - | `XAI_API_KEY` |
| Ganti permission | `permissions.defaultMode` | - |
| Tambah MCP server | `mcp.servers` | - |
| Override lokal | `.claw/settings.local.json` | - |
