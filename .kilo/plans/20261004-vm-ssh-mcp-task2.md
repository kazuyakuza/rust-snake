# Verification Plan — VM SSH MCP Server · Task 2: Verify the MCP server (20261004)

Source TODO: `.agent/todos/20261004/20261004-todo-1.md` → Task 2 (line 15).
Global plan (binding): `.kilo/plans/20261004-vm-ssh-mcp.md` (§ Technical & architecture decisions,
§ Config surface, § Task 2 spec highlights, § Security notes).
Task 1 artifacts (what was built): `.kilo/plans/20261004-vm-ssh-mcp-task1.md` and
`.kilo/plans/20261004-vm-ssh-mcp-task1-adherence.md`. Code under verification:
`C:\repo\vm-ssh-mcp`, branch `feat/mcp-server`, HEAD `8f602f1` (worktree clean).
Not front-end related → no 4.1a / 4.5a steps for this task.

[Project Info: Active]

---

## 0. Scope

**IN (Task 2 = verification only):**
- Determine VM state first (no MCP), then run the correct scenario branch.
- Run the official smoke test (`scripts/smoke-test.mjs` via `npm run smoke-test`) — mandatory in both branches.
- If the VM is running: run a **throwaway JSON-RPC driver** that additionally calls
  `vm_run_command` with `uname -a`, `docker ps`, and the disallowed `reboot`.
- Negative checks (password leak, stdout purity), evidence recording, tmp cleanup.

**OUT (do NOT do any of these):** code changes to `src/` or `scripts/`; edits to
`package.json` / `package-lock.json` / `.gitignore`; README (Task 4); opencode.jsonc
registration (Task 3); branch merge / push (Task 5); `[DONE]` marks (Step 4.6);
commits in either repo; TODO file edits; amending this plan file (implementer may not
edit `.kilo/plans/*` — Markdown Generation Rule).

**Defect rule (binding):** if any observed output contradicts the expectations in this
plan, do NOT fix anything, do NOT commit anything. Keep the tmp driver on disk as
evidence (skip Section 8 cleanup so it can be reproduced), STOP, and return the full
observation to the caller per Section 9.

**git safety (binding):** no branch creation/switch (Step 2 is done; stay on
`feat/mcp-server`); no version bump; no push (`vm-ssh-mcp` has no origin remote).
Zero commits are expected in Task 2 (the only exception is the caller-decided defect path).

---

## 1. Preconditions — verify as the very first actions (read-only)

Run from workdir `C:\repo\vm-ssh-mcp` unless stated otherwise:

| # | Command | Required result |
|---|---|---|
| P1 | `git -C "C:\repo\vm-ssh-mcp" status` | `On branch feat/mcp-server` / `nothing to commit, working tree clean` |
| P2 | `git -C "C:\repo\vm-ssh-mcp" log --oneline -1` | `8f602f1 refactor: simplify vm state parsing, exec handling and tool registration` |
| P3 | `git -C "C:\repo\vm-ssh-mcp" branch --show-current` | `feat/mcp-server` |
| P4 | `Test-Path -LiteralPath "C:\repo\vm-ssh-mcp\node_modules"` | `True` — if `False`, STOP and ask the caller (do NOT run `npm install`; Task 1 already installed) |
| P5 | `Test-Path -LiteralPath "C:\repo\vm-ssh-mcp\tmp"` | Expected `False` (fresh state). If `True`, note it in results and continue — the driver file is created fresh in Section 5.1 and Section 8 removes the whole `tmp\` tree |
| P6 | `Test-Path -LiteralPath "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"` | Informational; expected `True` (verify on 20261004: present) |
| P7 | `node -v` | Record the version in the results (Node ≥ 20 expected) |

Recon facts already verified by the planner on 20261004: node_modules present, `tmp\`
absent, VBoxManage.exe present, worktree clean at `8f602f1`, `.gitignore` contains only
`node_modules/`.

## 2. VM-state pre-probe (no MCP involvement) — do this FIRST

The MCP server itself must not be the only state oracle; classify the branch before
running anything else.

**Preferred probe** (bash tool, allows the `findstr` pattern, single pipeline, no chained
sub-commands):

```text
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" showvminfo "alpine-virt-3.21.2-x86_64" --machinereadable | findstr /C:"VMState"
```

- Expected shapes: `VMState="poweroff"` (or `saved`|`aborted`|`paused`) → branch OFFLINE;
  `VMState="running"` → branch LIVE. Record the raw observed line.
- **Permission advisory:** a sandbox may deny the call-operator invocation (`& "C:\Program
  Files\..."`). Retry the identical command once (tool rule: max 2 tries for a blocked
  command). If still denied or the output is empty/unparseable, use the **fallback
  classifier**: run the mandatory smoke test first (Section 3) and read the
  `vm_status -> ...` line — it prints `vmState: poweroff|running|unknown`. Record
  `pre-probe: via smoke-test only (VBoxManage invocation denied/unavailable)` in the results.
- `Test-Path` on the exe (P6) only proves the tool exists, never the VM state — it is
  reconnaissance only, never a branch decision.

**Branch decision:**

| Pre-probe observation | Branch |
|---|---|
| `VMState="running"` | LIVE (Section 3 + Section 5 driver) |
| `VMState="poweroff"` / other token / no such VM name | OFFLINE-ONLY (Section 3 only; live pass recorded as PENDING) |
| `unknown` / probe denied | classify after the smoke test's `vmState:` line; if it prints `poweroff`/`unknown` → OFFLINE-ONLY, if `running` → LIVE |

If the pre-probe says `running` but the smoke test prints `unknown` (or vice versa),
record both raw observations and treat the server output as authoritative — divergent
pre-probe results are reported, not fixed.

## 3. Mandatory smoke test (both branches)

Command (workdir `C:\repo\vm-ssh-mcp`): `npm run smoke-test`
(fallback if npm-script execution is denied: `node scripts/smoke-test.mjs`).

Mechanics recap (grounded in `scripts/smoke-test.mjs` at `8f602f1`): it spawns
`node src/server.mjs` with env `VM_SSH_PASSWORD ?? "alpine"` and
`VM_SSH_READY_TIMEOUT_MS ?? "4000"`, sends newline-delimited JSON-RPC `initialize` (id 1)
→ `notifications/initialized` → `tools/list` (id 2) → `tools/call vm_status` (id 3), and
has a 20 s guard that prints `SMOKE TEST TIMED OUT` and exits 1.

**Expected console with VM OFF (or unknown / no VM):**

```text
initialize -> serverInfo.name "vm-ssh-mcp" [PASS]
tools/list -> [vm_status, vm_run_command] [PASS]
vm_status  -> vmState: poweroff | sshReachable: false | VM 'alpine-virt-3.21.2-x86_64' is not running — start it in VirtualBox first. [PASS]
SMOKE TEST PASS
```

- If layer 1 yields `unknown` instead (VBoxManage probe failed/skipped), the third line is
  `vmState: unknown | sshReachable: false | Could not determine VM state — VBoxManage probe failed or was skipped.` — also a PASS.
- Em dashes (U+2014) may render as mojibake `?"/\xFFFD` in the console codepage — this is
  HARMBLESS (see `20261004-vm-ssh-mcp-task1-adherence.md` Appendix A: the source bytes are
  proper UTF-8). Never judge pass/fail by console rendering; judge by the `[PASS]` verdicts,
  the `SMOKE TEST PASS` line, and the exit code.

**Expected console with VM ON:**

```text
initialize -> serverInfo.name "vm-ssh-mcp" [PASS]
tools/list -> [vm_status, vm_run_command] [PASS]
vm_status  -> vmState: running | sshReachable: true | SSH reachable on 127.0.0.1:3022 — ready for commands. [PASS]
SMOKE TEST PASS
```

(Script stderr is inherited, so the server's single startup line
`vm-ssh-mcp ready on stdio — expect tools vm_status + vm_run_command` may appear
interleaved before these lines — harmless.)

**Pass criteria (both branches):** exit code 0; `initialize`/`tools/list`/`vm_status` all
`[PASS]`; final line `SMOKE TEST PASS`; no `SMOKE TEST FAIL`, no `SMOKE TEST TIMED OUT`,
no stack trace / uncaught-exception output.

**Edge state C1 — VM running but SSH unreachable:** the status text becomes
`vmState: running | sshReachable: false | VM 'alpine-virt-3.21.2-x86_64' is running but SSH is not reachable on 127.0.0.1:3022.`
The smoke-test regex `/not running|Could not determine|SSH reachable/` does NOT match this
text → the script prints `SMOKE TEST FAIL` and exits 1 even though the server behaved
correctly. If this exact edge occurs: do NOT "fix" anything; capture everything, STOP, and
report it as an observation (smoke-test acceptance gap; caller decides).

**If the smoke test fails in any other way / times out / crashes** → defect protocol (Section 9).

---

## 4. Policy message ground truth (from `src/vm-tools.mjs`, quoted from HEAD `8f602f1`)

The gate order in `vm_run_command` (`findGateFailure`): VM-state gate → SSH gate → policy
gate → execute. Exact strings (em dash = U+2014 everywhere below):

| ID / situation | Exact expected text | Response `isError` |
|---|---|---|
| status gate (VM off, tool call) | `VM not running — run vm_status for details and start the VM in VirtualBox first.` | `true` |
| SSH gate (VM on, SSH down) | `SSH not reachable on 127.0.0.1:3022.` | `true` |
| policy gate (disallowed) | `Command rejected: 'reboot' does not match allowed prefixes [docker, sh, apk, ls, cat, ps, df, free, uname, pwd, whoami].` | `true` |
| execution catch (post-start error) | `Command failed after start: <error.message>` | `true` |
| execution timeout | `Command timed out after <ms> ms` (inside the catch path) | `true` |
| success report | `exit code: <code>` line, then `stdout:` + block, then `stderr:` + block (10 000-char truncation with `… [truncated]`) | `true` iff `exit code != 0` |

`vm_status` report format (both tools use `{ content: [{ type: "text", text }], isError }`):

```text
vmState: <running|poweroff|unknown> | sshReachable: <true|false>
<message>
```

with message per state:
- poweroff → `VM 'alpine-virt-3.21.2-x86_64' is not running — start it in VirtualBox first.`
- unknown → `Could not determine VM state — VBoxManage probe failed or was skipped.`
- running + ssh up → `SSH reachable on 127.0.0.1:3022 — ready for commands.`
- running + ssh down → `VM 'alpine-virt-3.21.2-x86_64' is running but SSH is not reachable on 127.0.0.1:3022.`

Policy details (`src/command-policy.mjs`): match is case-sensitive, on the trimmed command;
allowed iff equal to a prefix or starts with `<prefix> ` (prefix + whitespace). The default
allowlist is `docker,sh,apk,ls,cat,ps,df,free,uname,pwd,whoami`; the rejection text joins
the prefixes with `", "` (comma + space). `reboot` is deliberately NOT allowed.

---

## 5. LIVE branch — throwaway JSON-RPC driver (only when VM state = running)

### 5.1 Create the throwaway file

1. If P5 was `False` (fresh state), create the directory (bash, workdir `C:\repo\vm-ssh-mcp`):
   `New-Item -ItemType Directory -Path "C:\repo\vm-ssh-mcp\tmp"`
2. Create `C:\repo\vm-ssh-mcp\tmp\live-driver.mjs` with the implementer's native `write` tool.
   - **Write-tool advisory:** if the native file tool refuses paths outside the workspace,
     do NOT improvise with shell redirection. STOP and ask the caller to create the file,
     providing the complete file content in your request.
   - The driver is a throwaway: it is NOT committed, and `.gitignore` stays untouched
     (do NOT add `tmp/` to `.gitignore` — that would be a committed change). Cleanup is
     by deletion in Section 8. It is exempt from the `.kilo` code-style rules (it is not
     committed), but keep it small (~150–250 lines), dependency-free
     (`node:child_process`, `node:process` only), and identical in style to
     `scripts/smoke-test.mjs` (same spawn + line-buffer mechanics).

### 5.2 Driver contract (binding)

The driver must run with workdir `C:\repo\vm-ssh-mcp` and speak exactly the same
newline-delimited JSON-RPC protocol as `scripts/smoke-test.mjs` (line JSON + `\n`; read
`child.stdout` with an `/\r?\n/` line splitter and a remainder buffer; index results by
`message.id`, treating `message.error` payloads like smoke-test's `recordResponse` does).

(a) **Server spawn** — replicate the smoke-test spawn (exact same shape):

```js
spawn(process.execPath, ["src/server.mjs"], {
  cwd: process.cwd(),
  env: {
    ...process.env,
    VM_SSH_PASSWORD: process.env.VM_SSH_PASSWORD ?? "alpine",
    VM_SSH_READY_TIMEOUT_MS: process.env.VM_SSH_READY_TIMEOUT_MS ?? "4000",
  },
  stdio: ["pipe", "pipe", "inherit"],
});
```

(b) **Requests — write all lines up-front (smoke-test pattern)**, each followed by \n:

```text
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"vm-ssh-mcp-live-driver","version":"0.1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"vm_status","arguments":{}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"vm_run_command","arguments":{"command":"uname -a"}}}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"vm_run_command","arguments":{"command":"docker ps"}}}
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"vm_run_command","arguments":{"command":"reboot"}}}
```

`reboot` (id 6) is the disallowed-command scenario: it must die at the policy gate BEFORE
being executed (gate order in `findGateFailure`: state → SSH → policy → exec), so the VM
must NOT actually reboot.

(c) **Assertions — run after the LAST id (6) arrives**, then print verdicts. Expected
observable values, grounded in Section 4 ground truth:

| id | Check | Expected (exact) | Pass criterion |
|---|---|---|---|
| any | JSON-RPC sanity | every parsed server stdout line has `jsonrpc === "2.0"` | counter `nonProtocolStdoutLines === 0` |
| any | no JSON-RPC errors | no id resolves to `{ jsonRpcError }` | `jsonRpcError === undefined` for every recorded id |
| 1 | initialize | `result.serverInfo.name === "vm-ssh-mcp"` and `serverInfo.version === "0.1.0"` | name+version equal |
| 2 | tools/list | includes both `vm_status` and `vm_run_command` (order not asserted) | both present |
| 3 | vm_status | first text line exactly `vmState: running \| sshReachable: true`; second line startsWith `SSH reachable on 127.0.0.1:3022` and endsWith `ready for commands.` | first line equal + prefix/suffix match |
| 4 | `uname -a` | first text line exactly `exit code: 0`; contains the `stdout:` section BEFORE a `stderr:` section; stdout content non-empty and startsWith `Linux`; `isError === false` | all five |
| 5 | `docker ps` | first text line exactly `exit code: 0`; `isError === false`; text does NOT contain `Command failed after start` and NOT `timed out` | faithful-report pass regardless of docker daemon/container state; record actual stdout/stderr for evidence |
| 6 | `reboot` | text equals byte-for-byte `Command rejected: 'reboot' does not match allowed prefixes [docker, sh, apk, ls, cat, ps, df, free, uname, pwd, whoami].`; `isError === true` | exact full-string equality |

(d) **Negative checks wired into the driver:**

1. `passwordLeak`: scan every recorded tool text and every stderr line for the literal
   `VM_SSH_PASSWORD` → must yield zero matches. (Do NOT search for bare `alpine` as a
   password indicator — the username equals the password, `whoami`/VM-name text contain
   `alpine` legitimately; only the env-var NAME leaking is a clear signal. The Task 1
   adherence report additionally verified the data-flow: the password only reaches ssh2
   connect options, never any log/text path.)
2. `stdoutProtocolOnly`: `nonProtocolStdoutLines === 0` (from (c)).
3. `stderrPurity`: the driver sets `stderr: "inherit"`; observe the merged console — expect
   exactly one startup line starting with `vm-ssh-mcp ready on stdio` and nothing further
   (no stack traces). Record any extra stderr lines verbatim in the results.

(e) **Guard + exit:** overall `GUARD_TIMEOUT_MS = 30000` via `setTimeout`; on fire print
`LIVE DRIVER TIMEOUT`, kill the child, exit 1. On completion print one
`id <n> <check-name> [PASS|<expected-vs-actual>]` line per check above, then
`LIVE DRIVER PASS` (all green → exit 0) or `LIVE DRIVER FAIL` (any red → exit 1). Always
`child.kill()` in a `finally`. Tolerate EPIPE on stdin writes after kill.

(f) **Blockchain of helper structure** (implementer fills the bodies; keep ≤ ~200 lines):

```js
// constants: GUARD_TIMEOUT_MS = 30000; EXPECTED_POLICY_TEXT = "..."; EXPECTED_STATUS_FIRST_LINE = "vmState: running | sshReachable: true";
// spawnServer()                       — literal (a) above
// createRequestLines()                — the exact 7 strings from (b)
// sendRequests(child)                 — write each line + "\n"; child.stdin.on("error", ...)
// createLineProcessor(onMessage)      — /\r?\n/ splitter + remainder, counts non-{"jsonrpc":"2.0"} lines
// evaluate(results)                   — per-id checks of table (c) + negative checks (d)
// printReport(report)                 — one verdict line per id + final LIVE DRIVER PASS/FAIL
// main()                              — await last-id arrival under 30000 ms guard; finally child.kill(); process.exit
```

Text-extraction helpers the implementer may mirror from `scripts/smoke-test.mjs`:
`recordResponse`, `createLineProcessor`, `processLines` semantics — but the driver adds a
non-JSON/non-protocol line counter.

### 5.3 Run + capture

Command (workdir `C:\repo\vm-ssh-mcp`): `node tmp\live-driver.mjs`
(permission advisory: node script execution beyond `node -v` may trigger a sandbox ask;
retry the identical command once; if still denied, STOP and ask the caller to grant or to
run it). Copy the full stdout/stderr output into the results. Record the exit code.

---

## 6. Verification matrix (consolidated)

| # | Scenario (pre-probe + smoke classification) | Exact commands (workdir `C:\repo\vm-ssh-mcp`) | Expected observable (ground truth §4) | Pass criteria | On failure |
|---|---|---|---|---|---|
| A | VM off — `VMState="poweroff"`/`saved`/`aborted`/`paused`, VM name missing, or probe `unknown` | `npm run smoke-test` | console per §3 "VM OFF" block: `vmState: poweroff\|unknown | sshReachable: false` + `... is not running — start it in VirtualBox first.` (or the `Could not determine` message) → `SMOKE TEST PASS`, exit 0 | exit 0 + all three `[PASS]` + token `poweroff` or `unknown` | full output capture → STOP (defect protocol) |
| A' | VM off through the TOOL (already covered by id 3 of smoke test; no separate run needed) | — n/a — | `vm_status` text = not-running message (smoke test's third line) | same as A | — |
| B | VM on — `VMState="running"` + smoke shows `sshReachable: true` | (1) `npm run smoke-test` (2) driver per §5: create tmp file, run `node tmp\live-driver.mjs` | smoke §3 "VM ON" block; driver: ids 1–6 per §5.2(c) table; `reboot` rejected with the exact policy text and NOT executed | smoke exit 0 + driver exit 0 + every driver check PASS + negative checks green | full output capture → STOP |
| C | VM on + disallowed command (`reboot`) — INSIDE branch B, driver id 6 | driver request line (b) id 6 | exact string `Command rejected: 'reboot' does not match allowed prefixes [docker, sh, apk, ls, cat, ps, df, free, uname, pwd, whoami].`, `isError === true`, and the VM stays up (no reboot side effect; subsequent vm_status would still be running — not asserted, the exact text is the criterion) | exact byte-equal text + isError true | full output capture → STOP |
| E | Edge — VM on but SSH not reachable (smoke prints `sshReachable: false` + "is running but SSH is not reachable") | `npm run smoke-test` | `SMOKE TEST FAIL` + exit 1 although the server behaved per spec (status regex gap in smoke-test) | NOT auto-fixable in this task → keep tmp artifacts if created, STOP, report the observation to the caller | caller decides |

Recording rule: do NOT fake a live pass. If the VM is off, record branch A and append
`live pass: PENDING (VM was off during Task 2 execution)` — this mirrors the global plan's
Task 2 wording.

## 7. Evidence recording (single source: the implementer's completion summary)

The markdown-generation rule forbids the Task-2 implementer from editing files under
`.kilo/plans/` — therefore results are NOT appended to this plan file. The implementer's
completion summary MUST end with the following filled template (verbatim header line,
`<...>` fields replaced with observed values; when a duty branch is skipped, write
`<journey: OFFLINE-ONLY>` style explicit values, never blanks):

```text
== TASK 2 VERIFICATION RESULTS (vm-ssh-mcp @ <head-sha> @ <execution date>) ==
pre-probe: <raw VMState line> | via-smoke-test-only
smoke-test: exit <code>; verdict SMOKE TEST <PASS|FAIL>; vmState <x>; sshReachable <bool>
branch: OFFLINE-ONLY | LIVE
driver: <created | not-created>; exit <code | n/a>
id1 initialize: [PASS] name "<name>" version "<v>"
id2 tools/list: [PASS] names <[...]>
id3 vm_status:  [PASS] line1 "<...>"; line2 start "<...>"
id4 uname -a:   [PASS] firstLine "exit code: 0"; stdout begin "<first 60 chars>"
id5 docker ps:  [PASS] firstLine "exit code: 0"; stdout summary "<...>"; stderr summary "<... or none>"
id6 reboot:     [PASS] exact-match yes; isError true
negative: passwordLeak <none>; nonProtocolStdoutLines <0>; stderr "<startup line + extras or none>"
cleanup: tmp removed <yes/no + reason>; git status clean <yes/no>; commits made <0>
defects: <none | full observation strings>
--- PRE-PROBE RAW OUTPUT ---
<paste>
--- SMOKE-TEST RAW OUTPUT ---
<paste>
--- DRIVER RAW OUTPUT ---
<paste or "not run (VM off)">
```

The caller (planner agent) persists the results wherever required later (e.g. the 4.5b
adherence report). No additional evidence file is created anywhere else.

## 8. Cleanup and final git verification (only when NO defect was found)

Run in order; each step records its observed result:

1. `Remove-Item -LiteralPath "C:\repo\vm-ssh-mcp\tmp" -Recurse -Force`
   (permission advisory: if the sandbox denies this cmdlet — it is outside the common
   allowlist — retry once; if still denied, STOP cleanup, keep the file, and record
   `cleanup: tmp removal blocked by permissions` in the summary; the caller resolves it.)
2. `Test-Path -LiteralPath "C:\repo\vm-ssh-mcp\tmp"` → `False` (directory must be entirely
   gone, not just emptied).
3. `git -C "C:\repo\vm-ssh-mcp" status` → `nothing to commit, working tree clean` (no
   untracked files, `node_modules/` still ignored).
4. `git -C "C:\repo\vm-ssh-mcp" log --oneline -1` → still `8f602f1 ...` (commits made = 0).
5. `git -C "C:\repo\vm-ssh-mcp" branch --show-current` → still `feat/mcp-server`.
6. Absolutely NO commit, NO merge, NO push, NO `[DONE]` mark, NO TODO edit, NO context.md
   edit in this step (those belong to later steps).

## 9. Defect protocol (binding)

Any of the following is a defect observation: server exits at startup (`vm-ssh-mcp failed
to start: ...`); smoke test prints `SMOKE TEST FAIL` or `SMOKE TEST TIMED OUT`, crashes, or
does not end with exit 0; driver: unexpected response for any id,
`jsonRpcError` present, `nonProtocolStdoutLines > 0`, password-leak hit, reboot NOT caught
at the policy gate (this would mean the policy is broken — treat as a security defect).
Actions: (1) preserve ALL raw console output; (2) if the driver was created before the
defect was observed, do NOT delete `tmp/` (keep it as reproduction evidence); (3) STOP —
return the observation text, reproduction steps, and the satisfied steps so far to the
caller. NO code fixes, NO commits, NO plan rewrites on the implementer side.

## 10. Compared to original task

TODO Task 2 text: "Verify the MCP server — offline smoke test (JSON-RPC `initialize`,
`tools/list`, `tools/call vm_status` with VM off → clean not-running result); if the Alpine
VM is running, live-test `vm_status` and `vm_run_command` (`uname -a`, `docker ps`)
through the server".

| TODO requirement | Covered by |
|---|---|
| offline smoke test `initialize` / `tools/list` / `tools/call vm_status` with VM off → clean not-running result | §3 (expected console, exit 0) + matrix A/A' |
| determine VM running state first | §2 pre-probe (VBoxManage `showvminfo ... --machinereadable | findstr VMState`, smoke-test fallback) |
| if VM running: live-test `vm_status` + `vm_run_command` (`uname -a`, `docker ps`) through the server | §5 driver ids 3/4/5 (JSON-RPC `tools/call`) |
| disallowed command scenario (VM on + `reboot`) | driver id 6 with exact policy-rejection assert (§5.2(c)) |
| password never in outputs; stdout stays protocol-only | §5.2(d) negative checks 1–3 |
| evidence recording | §7 (implementer summary template — chosen location) |
| cleanup, nothing committed in Task 2 | §8 (verified: HEAD unchanged, worktree clean) |
| extend beyond the built-in smoke test WITHOUT modifying committed code | driver is a throwaway file under `C:\repo\vm-ssh-mcp\tmp\`, deleted in §8; `.gitignore` NOT edited (cleanup-by-deletion chosen instead of a gitignore entry) |

Scope confirmation: verification only; NO code changes; NO README (Task 4); NO opencode
registration (Task 3); NO merges / `[DONE]` / push (Task 5); NO version bump; NO TODO edits.

## 11. Hand-off notes for later steps (do not execute now)

- 4.3 reviewer+simplifier: Task 2 produces no committed code — the review step records
  "not required (verification-only task; tmp driver deleted)" unless a defect fix is later
  authorized by the caller.
- 4.5b architector: verify the implementer summary against §6 matrix + §4 ground-truth
  strings + §8 commit/cleanup invariants; confirm `live pass: PENDING` (if the VM was off)
  is recorded honestly rather than turned into a pass.
- 4.6 implementer: append `[DONE]` to the Task 2 line in
  `.agent/todos/20261004/20261004-todo-1.md` (preserving the rest of the file), commit the
  TODO edit + this plan file in `C:\repo\rust-snake` on `feat/vm-ssh-mcp-orchestration`
  (suggested messages: `docs: mark vm ssh mcp task 2 verification complete` or analogous);
  no merges yet (Task 5).
- If the caller later authorizes a defect-fix cycle, it becomes its own plan +
  sub-task; Task 2 then RE-runs its verification from Section 1 after that fix.
