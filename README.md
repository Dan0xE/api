# 🛡️ CodeDefender

CodeDefender is a binary obfuscation platform designed to protect compiled programs against reverse engineering and static analysis. This repository contains the official Rust SDK and CLI tools for interacting with the CodeDefender SaaS backend.

---

## 📦 Crate Structure

This is a Cargo workspace with the following crates:

### `config/` → [`codedefender-config`](https://crates.io/crates/codedefender-config)  
Shared data structures and configuration types used by both the CLI and API client. This includes the `CDConfig` struct and the `AnalysisResult` model returned from the server.

### `api/` → [`codedefender-api`](https://crates.io/crates/codedefender-api)  
A blocking Rust client library for interacting with the CodeDefender SaaS backend. Upload binaries, perform analysis, trigger obfuscation, and poll for the obfuscated output.

### `cli/` → `codedefender-cli`  
A command-line tool built on top of `codedefender-api`, offering an easy way to run protection workflows locally from the terminal.

[Checkout the example here](cli/example/)

---