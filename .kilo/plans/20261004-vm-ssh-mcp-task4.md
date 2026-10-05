# Implementation Plan — VM SSH MCP Server · Task 4: Project documentation / README (20261004)

Source TODO: `.agent/todos/20261004/20261004-todo-1.md` → Task 4 (line 17):
"Document the project in `C:\repo\vm-ssh-mcp\README.md` — purpose, prerequisites,
configuration, tools, security notes, opencode integration, troubleshooting".
Global plan (binding): `.kilo/plans/20261004-vm-ssh-mcp.md` (§ Config surface,
§ Security notes, § Target structure, § Task 3 spec). Verification evidence source:
`.kilo/plans/20261004-vm-ssh-mcp-task2.md` (§4 ground truth, §6 matrix) plus the
recorded live results (smoke test PASS; live driver PASS: `uname -a` → Alpine
`6.12.8-0-virt`, `docker ps` → container table, `reboot` → byte-exact policy
rejection, no password leaks, stdout protocol-pure). Code base being documented:
`C:\repo\vm-ssh-mcp`, branch `feat/mcp-server`, HEAD `8f602f1` (worktree clean —
verified 20261004). Not front-end related → no 4.1a / 4.5a steps.

[Project Info: Active]

---

## 0. Executor, deliverable, and scope

- **Executor:** the **docs-specialist** sub-agent owns README creation and the
  documentation commit (Markdown Generation Rule: only planner/docs-specialist
  create/modify such files; implementer 4.2 is NOT used for this task).
- **Deliverable:** exactly one new file — `C:\repo\vm-ssh-mcp\README.md`.
- **Scope IN:** README content per §3; one commit in `C:\repo\vm-ssh-mcp` per §6.
- **Scope OUT:** any change to `src/`, `scripts/`, `package.json`,
  `package-lock.json`, `.gitignore`, the opencode.jsonc registration (Task 3
  already done — README only QUOTES it), TODO files, `[DONE]` marks (Step 4.6),
  branch merges / pushes (Task 5), any other documentation file
  (no `docs/` folder in this repo, no CHANGELOG, no LICENSE).

**Defect/blocking rule (binding):** if any ground-truth value in §2–§3 contradicts
what the docs-specialist reads in the code, STOP and return the observation to the
caller — do NOT silently "correct" the README away from the plan, and do NOT invent
values not present in this plan or the source.

## 1. Preconditions — verify as the very first actions (read-only)

Run from workdir `C:\repo\vm-ssh-mcp` unless stated otherwise:

| # | Action | Required result |
|---|---|---|
| P1 | `git status` | `On branch feat/mcp-server` / `nothing to commit, working tree clean` |
| P2 | `git log --oneline -1` | `8f602f1 refactor: simplify vm state parsing, exec handling and tool registration` |
| P3 | `Test-Path -LiteralPath "C:\repo\vm-ssh-mcp\README.md"` | `False` — README does not exist yet. If `True`, STOP and ask the caller (do not overwrite) |
| P4 | `node -v` | Informational (Node ≥ 20 expected; record the observed version) |

No `npm install`, no version bump (Step 3 is n/a: `package.json` exists at `0.1.0`;
a docs-only change is not a semver bump).

## 2. Ground truth (all README facts are copied from HERE, not improvised)

The docs-specialist must treat this section as the single source for every technical
value. Console output on this host can render U+2014 (em dash) as mojibake (`?"`,
`?`, `\uFFFD`) — the source files inside `C:\repo\vm-ssh-mcp` are proper UTF-8 and
their real strings use the em dash. If the specialist re-reads files to confirm,
quote from the files' logical content and this plan, NEVER from a mangled console
paste.

### 2.1 Configuration defaults (`src/vm-connection-config.mjs` — verified verbatim)

Server name: `vm-ssh-mcp`, version `0.1.0` (`src/server.mjs`, `McpServer` options).

| Env var | Default (exact) | Notes |
|---|---|---|
| `VM_SSH_HOST` | `127.0.0.1` | optional |
| `VM_SSH_PORT` | `3022` | validated: positive integer required |
| `VM_SSH_USER` | `alpine` | optional |
| `VM_SSH_PASSWORD` | none — **required** | missing/passwordless → server refuses to start |
| `VM_VBOX_NAME` | `alpine-virt-3.21.2-x86_64` | optional |
| `VM_VBOX_MANAGE_PATH` | unset ("" in code) → auto resolution | resolution order: (1) the env value if set; (2) `C:\Program Files\Oracle\VirtualBox\VBoxManage.exe` when that path exists; (3) `VBoxManage` resolved from PATH |
| `VM_ALLOWED_PREFIXES` | `docker,sh,apk,ls,cat,ps,df,free,uname,pwd,whoami` | comma-separated; each entry trimmed; empty entries dropped; matching is case-sensitive |
| `VM_COMMAND_TIMEOUT_MS` | `30000` | validated: positive integer required |
| `VM_SSH_READY_TIMEOUT_MS` | `5000` | validated: positive integer required |

Exact runtime error strings produced from these values (em dash = U+2014):

- Missing password → `VM_SSH_PASSWORD is required — set it in the MCP environment configuration`
- Integer validation failures → `<VAR> must be a positive integer, received '<value>'`
  (applies to `VM_SSH_PORT`, `VM_COMMAND_TIMEOUT_MS`, `VM_SSH_READY_TIMEOUT_MS`)
- Startup failure path (`src/server.mjs`) → stderr line
  `vm-ssh-mcp failed to start: <error message>` and exit code 1.
- Normal startup → one stderr line
  `vm-ssh-mcp ready on stdio — expect tools vm_status + vm_run_command`
  (stderr only; stdout is the MCP JSON-RPC channel and must stay pure).

### 2.2 Tool strings (`src/server.mjs`, `src/vm-tools.mjs` — verified verbatim)

Registered tool descriptions (quote in README "as is"):

- `vm_status` — "Check whether the Alpine VirtualBox VM is running and SSH reachable. ALWAYS call this before vm_run_command."
- `vm_run_command` — "Run one allowlisted shell command on the Alpine VM. The command must start with an allowed prefix (docker, sh, apk, ls, cat, ps, df, free, uname, pwd, whoami). Check vm_status first."

`vm_status` output shape (first line,`\n`,message), no arguments:

```text
vmState: <running|poweroff|unknown> | sshReachable: <true|false>
<message>
```

| State | `<message>` (exact, em dash) |
|---|---|
| poweroff (or saved/aborted/paused) | `VM 'alpine-virt-3.21.2-x86_64' is not running — start it in VirtualBox first.` |
| unknown | `Could not determine VM state — VBoxManage probe failed or was skipped.` |
| running + SSH up | `SSH reachable on 127.0.0.1:3022 — ready for commands.` |
| running + SSH down | `VM 'alpine-virt-3.21.2-x86_64' is running but SSH is not reachable on 127.0.0.1:3022.` |

`vm_run_command` result strings (exact):

| Situation | Text | `isError` |
|---|---|---|
| VM not running (status gate) | `VM not running — run vm_status for details and start the VM in VirtualBox first.` | true |
| VM running, SSH down | `SSH not reachable on 127.0.0.1:3022.` | true |
| Disallowed command | `Command rejected: '<command>' does not match allowed prefixes [<prefixes joined with ", ">].` (verified live for `reboot` → `Command rejected: 'reboot' does not match allowed prefixes [docker, sh, apk, ls, cat, ps, df, free, uname, pwd, whoami].`) | true |
| Execution error after start | `Command failed after start: <error.message>` | true |
| Timeout | `Command timed out after <ms> ms` | true |
| Execution completed | four-line report `exit code: <code>` / `stdout:` / stdout text / `stderr:` + stderr text; stdout/stderr capped at 10000 characters with `… [truncated]` appended when cut | true iff exit code ≠ 0 |

`vm_run_command` arguments (zod): `command` — string, required, minimum length 1;
`timeoutMs` — optional positive integer; defaults to `VM_COMMAND_TIMEOUT_MS`
(30000) when omitted.

### 2.3 Registered opencode block (verified verbatim — `C:\Users\ibej_\.config\opencode\opencode.jsonc`, "alpine-vm" entry inside the existing `mcp` object)

```jsonc
"alpine-vm": {
  "type": "local",
  "command": ["node", "C:\\repo\\vm-ssh-mcp\\src\\server.mjs"],
  "enabled": false,
  "environment": {
    "VM_SSH_HOST": "127.0.0.1",
    "VM_SSH_PORT": "3022",
    "VM_SSH_USER": "alpine",
    "VM_SSH_PASSWORD": "alpine",
    "VM_VBOX_NAME": "alpine-virt-3.21.2-x86_64",
    "VM_ALLOWED_PREFIXES": "docker,sh,apk,ls,cat,ps,df,free,uname,pwd,whoami"
  }
}
```

Context facts for the integration section: the global config already sets
`"mcp_timeout": 600000` under its `experimental` object; the registered block
supplies the password and all overrides, so enabling it is sufficient to run.
`enabled: false` → the server is registered but not started until flipped to
`true` and opencode is restarted (config is read at startup).

## 3. Required README structure (section-by-section requirements)

File: `C:\repo\vm-ssh-mcp\README.md`. Plain Markdown, headings `##` level for the
main sections, single `#` for the title. Expected length ~300–400 lines (well over
100 → TOC is mandatory, see §4). Section order and required content:

### 3.1 Title + purpose (`# vm-ssh-mcp`)

Exactly one paragraph immediately under the H1 covering, in this order: an MCP
(Model Context Protocol) stdio server that lets an AI agent check whether the local
Alpine Linux VirtualBox VM is running and run allowlisted shell commands on it over
SSH; built with Node.js (ESM) using `@modelcontextprotocol/sdk` and `ssh2`;
intended to be registered in opencode under the name `alpine-vm`; status-only — it
never starts or stops the VM. Do not add a tagline, badges, or logos.

### 3.2 Table of contents (`## Table of contents`)

One unordered list linking to every `##` section (anchor-style lowercase-hyphen
links). Must be updated to match the final section set; it may not reference
subsections.

### 3.3 Prerequisites (`## Prerequisites`)

Bullet list with exactly these entries:

- Node.js ≥ 20 (plain `.mjs` ESM — no build step)
- npm (dependencies install in-project: `npm install`; `node_modules/` is
  gitignored)
- Oracle VirtualBox installed so that `VBoxManage` is reachable (the server checks
  `C:\Program Files\Oracle\VirtualBox\VBoxManage.exe` first, then PATH; override
  with `VM_VBOX_MANAGE_PATH`)
- The target Alpine VM, running in VirtualBox, with sshd enabled and SSH
  port-forwarded to `127.0.0.1:3022`, user `alpine` (default VM name
  `alpine-virt-3.21.2-x86_64`; all of this is overridable via the §3.6 env vars)
- Docker installed inside the VM only if the agent should run `docker ...`
  commands (the allowlist contains `docker` by default)

### 3.4 How it works (`## How it works`)

Prose + optional small nested bullets covering exactly these facts:

1. **Two-layer state check** — Layer 1 asks VirtualBox itself:
   `VBoxManage showvminfo <name> --machinereadable` (5 s timeout, hidden window)
   and parses the `VMState=` line → `running` / `poweroff` (any of poweroff,
   saved, aborted, paused) / `unknown` (probe failure never crashes the server);
   Layer 2 opens a short SSH connection and runs `echo ok` expecting clean output —
   only executed when Layer 1 says the VM is running.
2. **`vm_status`** combines both layers into a single status report (§3.5 message
   table); **`vm_run_command`** applies three gates in order before anything
   executes: VM running → SSH reachable → command starts with an allowed prefix.
3. **Status-only** — there is no `VBoxManage startvm`/auto-start anywhere; when
   the VM is off, the tools return a clear result and the human starts the VM in
   VirtualBox.
4. One fresh SSH connection per command (`pty: false`), outputs capped at 10000
   characters, and the process timeout defaults to 30000 ms per command.
5. **stdout purity** — stdout carries only MCP JSON-RPC; all human logging (the
   single "ready" line, startup failures) goes to stderr.

### 3.5 Tools (`## Tools`)

- `### vm_status` — "Checks VM state; takes no arguments." Then an output/messages
  table: five rows, first column `State` (`poweroff`, `unknown`, `running, SSH
  up`, `running, SSH down`), second column `Reported text` containing the exact
  strings of §2.2 including the combined
  `vmState: … | sshReachable: …` first line format. Include two fenced `text`
  examples (VM off and VM ready), e.g.:

```text
vmState: poweroff | sshReachable: false
VM 'alpine-virt-3.21.2-x86_64' is not running — start it in VirtualBox first.
```

```text
vmState: running | sshReachable: true
SSH reachable on 127.0.0.1:3022 — ready for commands.
```

- `### vm_run_command` — argument table with two rows
  (`command` string required min length 1; `timeoutMs` positive integer optional,
  default `VM_COMMAND_TIMEOUT_MS` 30000) plus a "Gate order" line (VM running →
  SSH reachable → prefix allowlist). Then three fenced `text` examples:
  1. an allowed command — `uname -a` with the response starting `exit code: 0`,
     a `stdout:` section beginning `Linux` whose kernel line reflects the VM
     (`6.12.8-0-virt` was observed live), and an empty `stderr:` section (you may
     abbreviate the stdout tail with `...`);
  2. an allowed command — `docker ps` with `exit code: 0` and a `stdout:` section
     that begins with Docker's standard `CONTAINER ID   IMAGE   COMMAND   CREATED
     STATUS   PORTS   NAMES` header (describe the remaining rows in prose as the
     live container table rather than inventing container names);
  3. the disallowed `reboot` example with the byte-exact rejection string from
     §2.2 (this response arrives BEFORE execution — say so in the caption).

Any example marked "representative" must be labeled as such in the README prose;
verified strings (rejection, status messages) must not be labeled.

### 3.6 Configuration (`## Configuration`)

A three-column Markdown table (`Environment variable` / `Default` / `Purpose`)
listing all nine variables with the exact defaults of §2.1, ordered as in §2.1.
Below the table, add: (a) the validation note — numeric variables must be positive
integers, otherwise the server exits with
`<VAR> must be a positive integer, received '<value>'`; (b) the password note —
`VM_SSH_PASSWORD` has no default and the process refuses to start without it; (c)
the allowlist examples paragraph — `VM_ALLOWED_PREFIXES=docker,sh,apk` restricts
agents to those three prefixes; matching is case-sensitive and prefix-based (see
§3.9). No code blocks in this section.

### 3.7 Running with opencode (`## Running with opencode`)

1. State that the server is registered in the global opencode config
   `C:\Users\ibej_\.config\opencode\opencode.jsonc` under the existing `mcp`
   object as `alpine-vm`, and that it ships **disabled**.
2. Quote the exact JSONC block of §2.3 in a `jsonc` fence.
3. Enable instructions: open the file, change `"enabled": false` to `"enabled":
   true` inside this block only, save, and restart opencode (config is read at
   startup; `experimental.mcp_timeout` is already set globally).
4. One sentence: with the server enabled, the agent sees the two tools of §3.5
   and should call `vm_status` before `vm_run_command`.

### 3.8 Smoke test (`## Smoke test`)

- Command (PowerShell/cmd, workdir `C:\repo\vm-ssh-mcp`): `npm run smoke-test`
  (what it does: spawns `node src/server.mjs`, speaks newline-delimited JSON-RPC
  over stdio — `initialize` → `notifications/initialized` → `tools/list` →
  `tools/call vm_status` — under a 20 s guard).
- Two fenced `text` blocks, "Expected output — VM running" and "Expected output —
  VM off", exactly as verified in Task 2:

```text
initialize -> serverInfo.name "vm-ssh-mcp" [PASS]
tools/list -> [vm_status, vm_run_command] [PASS]
vm_status  -> vmState: running | sshReachable: true | SSH reachable on 127.0.0.1:3022 — ready for commands. [PASS]
SMOKE TEST PASS
```

```text
initialize -> serverInfo.name "vm-ssh-mcp" [PASS]
tools/list -> [vm_status, vm_run_command] [PASS]
vm_status  -> vmState: poweroff | sshReachable: false | VM 'alpine-virt-3.21.2-x86_64' is not running — start it in VirtualBox first. [PASS]
SMOKE TEST PASS
```

- Pass criteria sentence: exit code 0, three `[PASS]` verdicts, final
  `SMOKE TEST PASS`; add the one-line note that some Windows console codepages
  render the em dash as mojibake, which is harmless — judge by the `[PASS]`
  verdicts and the final line.
- One sentence on running the server by hand: `npm start` works once
  `VM_SSH_PASSWORD` is set in the environment (opencode supplies it via the
  registration block; interactive users must set it themselves).

### 3.9 Security notes (`## Security notes`)

Bullet list covering exactly:

- **Plaintext password rationale:** the password lives in plaintext inside the
  global opencode config. Acceptable for a disposable local development VM that
  never leaves the machine; knowingly NOT hardened — do not reuse this pattern
  on shared or production hosts.
- **Prefix allowlist behavior:** a command is allowed only if its trimmed form
  equals a configured prefix or starts with `<prefix> ` (prefix immediately
  followed by a space). Matching is case-sensitive. Consequence: `dockera` /
  `dockerx` variants are rejected, and a bare prefix with no arguments
  (`docker`) is allowed while `dockerX` is not. `reboot`, `rm`, `wget` etc. die
  at the policy gate BEFORE any execution (proven live with `reboot`).
- **Disposable-VM context:** the VM is a throwaway dev box; the allowlist limits
  blast radius but the box is treated as replaceable.
- **No auto privileges:** status-only; the server cannot start/reboot/stop the
  VM, and proxies nothing outside the allowlist.
- **Output hygiene:** the password is passed only to the ssh2 connect options —
  it never appears in tool results or logs (verified: zero leak hits), and
  stdout stays protocol-pure.
- **Future hardening path:** switch to SSH key authentication (ssh2 `privateKey`)
  and drop `VM_SSH_PASSWORD`; not implemented at version 0.1.0.

### 3.10 Troubleshooting (`## Troubleshooting`)

A two-column table (`Symptom` / `Cause and fix`) with at least these six rows,
strings as in §2.1–§2.2 (em dash included):

1. `vm_run_command` → `VM not running — run vm_status for details and start the VM in VirtualBox first.` → start the VM in VirtualBox, re-check with `vm_status`.
2. `vm_status` → `vmState: poweroff | sshReachable: false` + the `... is not running — start it in VirtualBox first.` message → VM is powered off (or saved/aborted/paused).
3. `vm_status` → `vmState: running | sshReachable: false` + `... is running but SSH is not reachable on 127.0.0.1:3022.` and `vm_run_command` → `SSH not reachable on 127.0.0.1:3022.` → sshd is down or the port forwarding changed — check the VM service and the 127.0.0.1:3022 forward.
4. Server exits immediately; stderr shows `vm-ssh-mcp failed to start: VM_SSH_PASSWORD is required — set it in the MCP environment configuration` → set the password in the environment (the registered opencode block already carries it).
5. Server exits with `<VAR> must be a positive integer, received '<value>'` → fix the numeric env value.
6. `vm_status` → `vmState: unknown | sshReachable: false` + `Could not determine VM state — VBoxManage probe failed or was skipped.` → VBoxManage not found/VM name wrong: set `VM_VBOX_MANAGE_PATH` and/or `VM_VBOX_NAME`.
7. Optional row: `Command rejected: ...` → the command's first word is not an allowed prefix — extend `VM_ALLOWED_PREFIXES` deliberately, or pick an allowed command.
8. Optional row: `Command timed out after <ms> ms` → pass a larger `timeoutMs` for that call, or raise `VM_COMMAND_TIMEOUT_MS`.

### 3.11 Project layout (`## Project layout`)

A fenced `text` tree followed by a short purpose table (or bullet list) — one
line per file, exactly:

```text
C:\repo\vm-ssh-mcp\
├── package.json
├── package-lock.json
├── .gitignore
├── README.md
├── scripts\
│   └── smoke-test.mjs
└── src\
    ├── server.mjs
    ├── vm-connection-config.mjs
    ├── vm-state-probe.mjs
    ├── ssh-command-runner.mjs
    ├── command-policy.mjs
    └── vm-tools.mjs
```

- `package.json` — npm manifest: `vm-ssh-mcp` 0.1.0, `type: module`; scripts
  `start` (`node src/server.mjs`) and `smoke-test`; deps
  `@modelcontextprotocol/sdk`, `ssh2`, `zod`.
- `package-lock.json` — locked dependency versions for in-project installs.
- `.gitignore` — ignores `node_modules/`.
- `README.md` — this document.
- `scripts/smoke-test.mjs` — JSON-RPC-over-stdio smoke test (spawns the server).
- `src/server.mjs` — entrypoint: `McpServer` + `StdioServerTransport`; registers
  both tools; startup/errors logged to stderr only.
- `src/vm-connection-config.mjs` — reads/validates the nine `VM_*` env vars into
  a frozen config object; throws early on a missing password or invalid integer.
- `src/vm-state-probe.mjs` — Layer-1 probe: `VBoxManage` `showvminfo
  --machinereadable` → `running` | `poweroff` | `unknown` (never throws).
- `src/ssh-command-runner.mjs` — Layer-2 SSH: one `ssh2` client per command;
  `exec` → `{stdout, stderr, exitCode}`; exports the `echo ok` probe used for
  SSH reachability.
- `src/command-policy.mjs` — pure prefix-allowlist matcher (equal or `<prefix> `
  prefix-slash rule, case-sensitive, trimmed command).
- `src/vm-tools.mjs` — the two tool handlers: state gate → SSH gate → policy
  gate → execute + format (10000-char truncation).

### 3.12 Verification record (`## Verification record`)

One short paragraph + optional dated list. Required content, honestly stated:

- Verified on 2026-10-04 against commit `8f602f1` (branch `feat/mcp-server`),
  server `vm-ssh-mcp` 0.1.0.
- Offline smoke test PASS (JSON-RPC `initialize` / `tools/list` / `tools/call
  vm_status` with the VM off; exit 0).
- Live pass with the Alpine VM running: `vm_status` reported
  `vmState: running | sshReachable: true`; `uname -a` returned the Alpine kernel
  line (`6.12.8-0-virt`); `docker ps` returned Docker's container table with
  exit code 0; the disallowed `reboot` was rejected with the byte-exact policy
  string of §2.2 and never executed.
- Negative checks: no password leakage in any tool output or stderr, and the
  server stdout carried only JSON-RPC lines.

## 4. Style constraints (README)

1. Plain Markdown only — no HTML, no images, no front-matter.
2. No emojis, no decorative characters.
3. No commented-out content anywhere (including inside code fences: a `jsonc`
   block may only contain JSONC structure as in §2.3, no `//` commentary).
4. **TOC required** — the file will exceed 100 lines (workflow 4.4 rule); keep it
   in sync with headings.
5. Every fenced code block MUST carry a language tag (`text`, `jsonc`); terminal
   command lines may live in `text` fences or inline backticks.
6. Real newlines: the file must be written with actual CR/LF breaks (the writing
   strategy preserves them — see §5). Never emit the two-character sequence
   `\n` as visible text.
7. Em dashes (U+2014) inside quoted strings are the real characters; if a console
   verification shows mojibake, that is a console artifact — verify with a byte/
   pattern check per §7, not the eye.
8. No invented facts: every string must trace to §2 or the source files; mark
   representative examples as "representative" in prose.
9. Hard line width is not enforced; prefer ≤ ~120 chars for readability.

## 5. File creation mechanics (workdir is `C:\repo\rust-snake`; target is cross-repo)

The target path lives outside the workspace tool restrictions, so use this exact
ladder (stop as soon as one step succeeds), then verify:

1. **Primary:** use the native `write` tool with the absolute path
   `C:\repo\vm-ssh-mcp\README.md`, composing the full content from §3. (If the
   tool accepts the path, skip to the verify step.)
2. **Fallback A:** write the full content to the pre-approved temp directory
   `C:\Users\ibej_\AppData\Local\Temp\opencode\vm-ssh-mcp-readme.md` with the
   native `write` tool, then copy it into place in a SINGLE bash call:
   `Copy-Item -LiteralPath "C:\Users\ibej_\AppData\Local\Temp\opencode\vm-ssh-mcp-readme.md" -Destination "C:\repo\vm-ssh-mcp\README.md" -Force`
3. **Fallback B:** if both are denied, use one bash call that writes the file via
   .NET (UTF-8, real newlines preserved by the here-string):
   `[System.IO.File]::WriteAllText("C:\repo\vm-ssh-mcp\README.md", @'
   <full README content between the here-string markers>
   '@)`.
4. **If all three are denied:** STOP and report the blocker to the caller (do NOT
   improvise other write locations and do NOT create the file inside the
   `rust-snake` workspace).

Verification-after-write (kill any encoding/newline issue immediately):

- `Test-Path -LiteralPath "C:\repo\vm-ssh-mcp\README.md"` → `True`.
- File reads back cleanly: first line is `# vm-ssh-mcp`.
- Real-newline check: the raw file has zero visible `\n` two-character
  sequences at line starts (grep pattern `^\s*\\n` must not match).
- Em-dash integrity check: a byte-level search for the em-dash character finds
  matches (e.g., `Select-String -LiteralPath "C:\repo\vm-ssh-mcp\README.md" -Pattern [char]0x2014`
  returns hits; expect ≥ 1). If zero hits AND the written strings contained em
  dashes, re-write the file (an encoding mangle occurred).

## 6. Git actions (single commit, in `C:\repo\vm-ssh-mcp` only)

After §5 verification passes, run from workdir `C:\repo\vm-ssh-mcp`:

1. `git status` → must show exactly one untracked/modified entry: `README.md`.
   Read `.gitignore` first (contains only `node_modules/`) — README.md is fine to
   stage; `node_modules/` must not appear.
2. `git add README.md`
3. `git commit -m "docs: add project readme"` — exact message, no trailer, no
   co-author lines.
4. Verify: `git log --oneline -2` → the new `docs: add project readme` commit
   sits directly on top of `8f602f1`; `git status` → `nothing to commit, working
   tree clean`; `git branch --show-current` → still `feat/mcp-server`.
5. NO push (`vm-ssh-mcp` has no origin remote), NO merge, NO branch creation, NO
   version bump, NO TODO edits in this step (Step 4.6 owns the `[DONE]` mark in
   `C:\repo\rust-snake`).

## 7. Self-check checklist (docs-specialist, before returning the summary)

Answer each item explicitly in the completion summary; any `No` → fix the README
before committing or STOP and ask the caller.

1. All twelve parts of §3 present in the given order, headings as specified.
2. TOC present and every link resolves to a real heading.
3. Configuration table has exactly the nine variables of §2.1 with exact
   defaults (spot-check: `VM_SSH_PORT` default `3022`,
   `VM_COMMAND_TIMEOUT_MS` default `30000`, `VM_SSH_READY_TIMEOUT_MS` default
   `5000`, allowlist string joined with commas and no spaces).
4. The registered JSONC block is byte-identical to §2.3 (including
   `"enabled": false` and double backslashes in the command path).
5. Exactly the two tools (`vm_status`, `vm_run_command`) with argument tables and
   the three `vm_run_command` examples of §3.5; rejection string byte-exact.
6. All quoted message strings match §2.1–§2.2 (em dash present; no mojibake).
7. Security bullets include the prefix+space rule, plaintext-password rationale,
   disposable-VM context, and SSH-key future hardening.
8. Troubleshooting table includes the required four symptoms (VM off; sshd down;
   missing `VM_SSH_PASSWORD`; VBoxManage not found) plus validation and reject
   rows.
9. Project layout tree + per-file one-liners match §3.11.
10. Verification record dated 2026-10-04 and tied to commit `8f602f1`; no fake
   /extemporized evidence (only Task 2 results are cited).
11. Style: no emojis, no HTML, no commented-out content, language tags on all
    fences, real newlines, plain Markdown.
12. File length > 100 lines (hence TOC mandatory) — report the line count.
13. Exactly one new commit `docs: add project readme` atop `8f602f1`; worktree
    clean; branch `feat/mcp-server` untouched.

## 8. Completion summary template

The docs-specialist's summary must end with this filled form (`<...>` replaced;
never leave blanks):

```text
== TASK 4 README RESULTS (vm-ssh-mcp @ feat/mcp-server @ <execution date>) ==
preconditions: branch <feat/mcp-server>; HEAD <8f602f1>; README existed <no>
write strategy used: <primary | fallback A | fallback B>
file: C:\repo\vm-ssh-mcp\README.md; lines <n>; sections <12>; TOC <yes>
em-dash integrity: <hits found / n>
self-check: <13 items — all yes / list of No items fixed>
commit: <hash> "docs: add project readme"
post-state: worktree clean <yes>; HEAD <new sha>; branch <feat/mcp-server>
NOT done: <no code changes; no other docs; opencode.jsonc untouched; no push; no merge; no TODO edits>
blockers: <none | full description>
```

## 9. Compared to original task

TODO Task 4 text: "Document the project in `C:\repo\vm-ssh-mcp\README.md` —
purpose, prerequisites, configuration, tools, security notes, opencode
integration, troubleshooting".

| TODO requirement | Covered by |
|---|---|
| purpose | §3.1 (H1 + one paragraph) |
| prerequisites | §3.3 (Node ≥ 20, npm, VirtualBox/VBoxManage, Alpine VM with sshd on 127.0.0.1:3022) |
| configuration | §3.6 (all nine `VM_*` vars, exact defaults from `src/vm-connection-config.mjs`) |
| tools | §3.5 (`vm_status`, `vm_run_command`, argument tables, examples) |
| security notes | §3.9 (plaintext password rationale, allowlist incl. prefix+space rule, disposable-VM context, SSH-key future hardening) |
| opencode integration | §3.7 (exact registered JSONC block; `enabled:false` → flip true; restart) |
| troubleshooting | §3.10 (VM off; sshd down; missing `VM_SSH_PASSWORD`; VBoxManage not found; extras) |
| beyond-TODO additions grounded in the global plan/workflow 4.4 | §3.2 TOC (mandatory >100 lines), §3.4 how-it-works, §3.8 smoke test, §3.11 project layout, §3.12 verification record (Task 2 evidence with date) |

Scope confirmation: README only — NO code changes, NO other documentation files,
NO opencode.jsonc modifications (it is only quoted), NO merges, NO pushes, NO
TODO edits, NO version bump. Single commit `docs: add project readme` on
`feat/mcp-server` in `C:\repo\vm-ssh-mcp`.

## 10. Hand-off notes for later steps (do not execute now)

- Step 4.3 (code-reviewer/code-simplifier): not required — no committed source
  change in Task 4 (README is a directory-listing-level artifact, not code);
  record "not applicable (docs-only)".
- Step 4.5b (architector adherence): compare the actual README against §3.1–§3.12
  and §4; confirm the registered block quoted in the README still matches
  `C:\Users\ibej_\.config\opencode\opencode.jsonc` verbatim; confirm em dashes
  and newline integrity per §5; confirm exactly one docs commit atop `8f602f1`.
- Step 4.6 (implementer): append `[DONE]` to the Task 4 line (line 17) of
  `.agent/todos/20261004/20261004-todo-1.md` preserving all other content, then
  commit the TODO edit + this plan file in `C:\repo\rust-snake` on
  `feat/vm-ssh-mcp-orchestration`.
- Task 5: merges `feat/mcp-server` (now containing the README) into `main` in
  `C:\repo\vm-ssh-mcp` — no remote push (notify user instead).
