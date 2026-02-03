# Safe Smoke Test Plan

**Goal:** Verify hardening without using real credentials or connecting to production APIs.

## 1. Local Development Check (Requires Node + Rust)

1. **Start the App**:

    ```powershell
    npm run tauri dev
    ```

2. **Verify No Update Check**:
    - Watch the terminal output.
    - Confirm you see: `[INFO] Update check disabled (safe mode).`
    - Verify NO log says `Checking for new version from GitHub...`.
3. **Verify Network Binding**:
    - Open a new terminal.
    - Run: `netstat -ano | findstr 8045`
    - **Pass**: Result is `127.0.0.1:8045`.
    - **Fail**: Result is `0.0.0.0:8045` (unless you set `ALLOW_LAN=1`).

## 2. Headless / Docker Simulation

You can simulate the headless mode locally by running the binary directly with `ALLOW_LAN` settings.

1. **Build (if not already)**:

    ```powershell
    npm run tauri build
    ```

    *(Or just `cargo build --release` in `src-tauri`)*

2. **Test Secure Default (Localhost)**:
    - Run: `./src-tauri/target/release/antigravity-manager.exe` (path may vary)
    - Check `netstat` again. Should be `127.0.0.1`.

3. **Test Opt-In LAN (0.0.0.0)**:
    - Run:

      ```powershell
      $env:ALLOW_LAN="1"; ./src-tauri/target/release/antigravity-manager.exe
      ```

    - Check `netstat`. Should be `0.0.0.0`.

## 3. Runtime Safety

- **Secrets**: NEVER put real API keys in `tauri.conf.json` or `config.json`.
- **Use Env Vars**:

  ```powershell
  $env:API_KEY="dummy-key-for-testing"
  npm run tauri dev
  ```

## 4. Docker Example (Localhost Only)

```bash
# Secure default (binds to container localhost, might need port mapping to access)
docker run -d --name antigravity-manager \
  -p 127.0.0.1:8045:8045 \
  -e API_KEY=dummy \
  -v ~/.antigravity_tools:/root/.antigravity_tools \
  lbjlaq/antigravity-manager:latest

# Note: If you need to access it from host, you might actually NEED ALLOW_LAN=1 inside the container
# if the container process binds to 127.0.0.1 inside the container.
# The hardening makes the PROCESS bind 127.0.0.1. Inside Docker, this means it's unreachable from outside.
# So for Docker, you MUST pass -e ALLOW_LAN=1.
docker run -d --name antigravity-manager \
  -p 127.0.0.1:8045:8045 \
  -e API_KEY=dummy \
  -e ALLOW_LAN=1 \
  -v ~/.antigravity_tools:/root/.antigravity_tools \
  lbjlaq/antigravity-manager:latest
```

*Note: We bind the Docker container port `-p 127.0.0.1:8045:8045` so the container is only accessible from the host machine, not the whole network.*
