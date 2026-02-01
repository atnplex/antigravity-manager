# Hardening Report

**Base Commit:** 003932150be7fa601fe5520b9bffc93ad6e54518
**Branch:** harden/no-telemetry-no-updater

## Planned Hardening Steps

1. [x] Disable Tauri updater in `tauri.conf.json`.
2. [x] Remove updater plugin from Rust backend (`Cargo.toml` & code).
3. [x] Remove updater plugin from frontend (`package.json` & code).
4. [x] Analyze and gate telemetry/device fingerprinting.
5. [x] Inventory external endpoints.
6. [ ] Build and smoke test.

## Execution Log

### Phase A: Static Audit

- Identified updater vectors in config, Rust, and Frontend.
- Found device fingerprinting logic in `src-tauri/src/modules/device.rs`.
- Located external API endpoint constants.

### Phase B: Hardening Edits

- [x] **Step 1: Disable Tauri updater in config (`tauri.conf.json`)**.
  - Set `active` to `false`.
  - Cleared `endpoints` and `pubkey`.
- [x] **Step 2: Remove updater plugin from Rust backend**.
  - Removed `tauri-plugin-updater` from `Cargo.toml`.
  - Removed plugin init from `lib.rs`.
  - Disabled remote version check in `constants.rs` (using Cargo fallback).
  - Disabled manual update check logic in `modules/update_checker.rs`.
- [x] **Step 3: Remove updater plugin from frontend (`package.json`, `UpdateNotification.tsx`, etc.)**.
  - Removed `@tauri-apps/plugin-updater` from `package.json`.
  - Removed imports and usages in `UpdateNotification.tsx`.
- [x] **Step 4: Telemetry/device fingerprinting sanity pass**.
  - **Findings**: `src-tauri/src/modules/device.rs` manages local device IDs (`machineId`, `sqmId`) in `storage.json` for VS Code compatibility. No external analytics SDKs (Sentry, PostHog, etc.) were found in dependencies.
  - **Verdict**: Safe (Functional logic only).
- [x] **Step 5: Inventory external endpoints**.
  - **Endpoints Found**:
    - `https://api.z.ai/` (Anthropic proxy)
    - `https://cloudcode-pa.googleapis.com` (Google Cloud Code)
    - `https://generativelanguage.googleapis.com` (Gemini)
    - `https://api.anthropic.com`, `https://api.openai.com` (CLI Sync mappings)
    - `https://api.github.com` (Update check - logic disabled)
    - `https://oauth2.googleapis.com`, `https://accounts.google.com` (OAuth)
  - **Action Taken**: Default proxy bind address changed from `0.0.0.0` (implied `allow_lan_access=true` in headless) to `127.0.0.1` (`allow_lan_access=false`) in `src-tauri/src/lib.rs` to prevent accidental exposure.

### Phase C: Build & Verify

- [ ] Run `npm install` to clean up dependencies.
- [ ] Run `npm run tauri build` (or backend check) to ensure no compilation errors.
- [ ] Verify functionality (smoke test).
