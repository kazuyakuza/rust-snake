# Code Review Report — VM SSH MCP Server · Task 1

**Review date:** 2026-10-04  
**Code under review:** `C:\repo\vm-ssh-mcp` (branch `feat/mcp-server`)  
**Baseline:** `.kilo/plans/20261004-vm-ssh-mcp-task1.md` Steps 0–12 + Section 5 checklist, and `.kilo/plans/20261004-vm-ssh-mcp.md`  
**Reviewer:** code-reviewer sub-agent (Task 1, step 4.3)

---

## Summary

The implementation follows the architecture, env contract, gate order, and tool wiring defined in the plan. `npm run smoke-test` passes (VM was observed running during this review). Git history, branch state, package contents, and `.gitignore` match the plan.

There are **no BLOCKERs** and no security bypasses beyond the intentionally allowlisted `sh` prefix. The required fixes are:

1. Several user-facing strings use an ASCII hyphen (`-`) where the plan specifies an em dash (`—`), and the truncation suffix uses `.` instead of the ellipsis (`…`) specified by the plan.
2. `scripts/smoke-test.mjs` violates the `max-depth.md` rule (3 levels of nested callback blocks in `createLineProcessor`).

These are documented below with concrete fix instructions.

---

## Findings

### MAJOR — Message texts deviate from the exact plan strings (punctuation)

The plan specifies em dashes (`—`, U+2014) in several user-facing strings. The implementation uses ASCII hyphens (`-`). The plan also specifies the truncation suffix `… [truncated]` (U+2026); the implementation uses `. [truncated]`.

| File | Line | Current text | Expected text |
|---|---|---|---|
| `src/vm-connection-config.mjs` | 27 | `VM_SSH_PASSWORD is required - set it...` | `VM_SSH_PASSWORD is required — set it...` |
| `src/vm-tools.mjs` | 14 | `...ready for commands.` | `...— ready for commands.` |
| `src/vm-tools.mjs` | 25 | `...is not running - start it...` | `...is not running — start it...` |
| `src/vm-tools.mjs` | 32 | `...VM state - VBoxManage probe...` | `...VM state — VBoxManage probe...` |
| `src/vm-tools.mjs` | 50 | `VM not running - run vm_status...` | `VM not running — run vm_status...` |
| `src/server.mjs` | 37 | `ready on stdio - expect tools...` | `ready on stdio — expect tools...` |
| `src/vm-tools.mjs` | 66 | `${output.slice(0, OUTPUT_LIMIT)}. [truncated]` | `${output.slice(0, OUTPUT_LIMIT)}… [truncated]` |

**Fix instruction:** Replace the ASCII hyphen separator with the Unicode em dash character (`—`) in all six strings, and replace the truncation marker `.` with the Unicode ellipsis character (`…`) in `truncateOutput`. Verify with `Select-String` or a hex check that the source files contain `0xE2 0x80 0x94` for the dash and `0xE2 0x80 0xA6` for the ellipsis.

---

### MAJOR — `scripts/smoke-test.mjs` exceeds the maximum nesting depth

`max-depth.md` requires no more than 2 levels of nested blocks. `createLineProcessor` at `scripts/smoke-test.mjs:49-56` nests a returned arrow function, a `.filter()` callback, and a `.forEach()` callback, producing 3 levels of block nesting from the top of the function.

**Fix instruction:** Refactor `createLineProcessor` so the returned function stays at depth ≤2. For example:

- Extract `recordIfNonEmpty(results, line)` that contains the single `if` guard and the `recordResponse` call.
- Extract `processLines(results, lines)` that iterates `lines` and calls `recordIfNonEmpty`.
- The returned arrow function should only compute `lines`, update `remainder`, and call `processLines(results, lines)`.

This keeps each helper at ≤2 nesting levels and preserves the current behavior.

---

### MINOR — Smoke test silently ignores JSON-RPC error responses

`recordResponse` at `scripts/smoke-test.mjs:67-73` only records messages that contain a `result` field. If the server returns a JSON-RPC error object (with `error` and no `result`), the test will eventually time out or report a generic failure instead of surfacing the error.

**Fix instruction:** Extend `recordResponse` to also store/error-out on messages that contain `error`, or add an explicit `error` check that fails the test with the error payload. This is a diagnostic improvement, not a protocol violation.

---

## OK — Compliance / contract items verified

| Item | Status | Notes |
|---|---|---|
| APIs / module shape | OK | Matches the exports and signatures in Steps 2–7. |
| Gate order in `vm_run_command` | OK | Status → SSH → policy → execute, per Step 6. |
| Env contract / defaults / validation | OK | `createVmConnectionConfig` uses the specified defaults, throws on missing password, and freezes the config. |
| Timeouts | OK | Command 30000 ms, SSH ready 5000 ms, VBoxManage probe 5000 ms. |
| Commit structure | OK | `feat/mcp-server` has scaffold + 7 granular feature/test commits; tree is clean. |
| ssh2 stream handling | OK | Uses `close` for exit code and maps `null` to `1`; connect errors mapped. |
| `execFile` VMState parsing | OK | Case-insensitive `vmstate=` lookup, quote stripping, token mapping, errors → `"unknown"`. |
| MCP SDK usage | OK | `McpServer`, `StdioServerTransport`, `registerTool` signature, and zod schema match; smoke test confirms. |
| Smoke-test JSON-RPC protocol | OK | `initialize` → `notifications/initialized` → `tools/list` → `tools/call` in order. |
| Password handling | OK | Password is read from env and passed to ssh2; never logged or returned. |
| Allowlist bypass vectors | OK | Leading whitespace is trimmed; prefix must be followed by a literal space (not tab), matching the plan’s `<prefix> ` rule. |
| No `startvm` / auto-start | OK | No `VBoxManage startvm` or equivalent exists anywhere. |
| stdout purity | OK | No `console.log` in `src/`; server logs to `stderr` only. |
| File line counts | OK | All `.mjs` files ≤200 lines (largest is `scripts/smoke-test.mjs` at 131 lines). |
| Function body lengths | OK | No function body exceeds 50 lines. |
| Parameter counts | OK | No function has >2 params; multi-property args are bundled in single config/result objects. |
| `if`/`while` conditions | OK | No `if`/`while` uses compound `&&`/`||`; all are single-section. (Return-expression `&&` in predicates matches the plan’s `hasAllowedPrefix` / `isCleanEcho` snippets.) |
| Comments / commented-out code | OK | None found in `src/` or `scripts/`. |
| Private-by-default exports | OK | Only the planned public exports are present. |
| Implementer restriction | OK | No scope expansion: README, live verification, opencode.jsonc registration, and merges were not touched. |

---

## Fix Plan Path

`C:\repo\rust-snake\.kilo\plans\20261004-vm-ssh-mcp-task1-review-fix.md`

**What was done:** Read the TODO, global plan, task plan, all source files, package metadata, `.gitignore`, and git history; ran `npm run smoke-test`; checked rule compliance manually.

**What was NOT done:** No code was modified. The fixes above must be applied by the implementer in a follow-up step.
