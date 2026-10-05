# Task 4 README Review Fix Plan — vm-ssh-mcp

- **Artifact reviewed:** `C:\repo\vm-ssh-mcp\README.md`
- **Reviewer:** code-reviewer
- **Review date:** 2026-10-04
- **Artifact commit:** `c6904de` (`docs: add project readme`) on `feat/mcp-server`
- **Ground-truth source:** `C:\repo\vm-ssh-mcp\src\*.mjs`, `scripts/smoke-test.mjs`, `package.json`, `C:\Users\ibej_\.config\opencode\opencode.jsonc`
- **Plan contract:** `C:\repo\rust-snake\.kilo\plans\20261004-vm-ssh-mcp-task4.md`
- **Verification facts:** `C:\repo\rust-snake\.kilo\plans\20261004-vm-ssh-mcp-task2.md`

## Verdict

**MAJOR — fixes required.** The README is structurally complete and most strings match the source, but it contains one incorrect documented output punctuation (`… [truncated]` vs `. [truncated]`) and two minor presentation deviations. A fix pass is needed before the documentation can be considered accurate.

## Findings

### 1. MAJOR — Wrong truncation punctuation (`README.md:36`)

- **Location:** `## How it works`, bullet 4, line 36.
- **Current text:** "stdout and stderr capped at 10000 characters each with `. [truncated]` appended when cut"
- **Ground truth:** `src/vm-tools.mjs:66` appends the horizontal ellipsis character (U+2026) followed by a space and `[truncated]`:
  ```js
  return `${output.slice(0, OUTPUT_LIMIT)}… [truncated]`;
  ```
  Plan §2.2 also specifies `… [truncated]`.
- **Impact:** The documented truncation suffix is not byte-exact; a reader copying the literal `. [truncated]` string would not match actual tool output.
- **Fix:** Replace the backtick content on line 36 with `… [truncated]` (use U+2026, not three separate periods).

### 2. MINOR — Extra `vm_status` table row with escaped pipes (`README.md:47`)

- **Location:** `## Tools` > `### vm_status` message table, line 47.
- **Current text:** `| First line, any state | \`vmState: <running\|poweroff\|unknown> \| sshReachable: <true\|false>\` |`
- **Ground truth / plan:** Plan §3.5 requires only four state rows (`poweroff`, `unknown`, `running, SSH up`, `running, SSH down`). The first-line format is described by the fenced examples and by §2.2. The `\|` escapes render as literal backslashes in the displayed text, so the row is not byte-exact either.
- **Impact:** Low — extra, slightly malformed row; does not contradict other sections.
- **Fix:** Delete line 47. The first-line format remains covered by the two fenced `text` examples directly below.

### 3. MINOR — `docker ps` example header spacing (`README.md:94`)

- **Location:** `## Tools` > `### vm_run_command`, `docker ps` representative example, line 94.
- **Current text:** `CONTAINER ID   IMAGE   COMMAND   CREATED STATUS   PORTS   NAMES`
- **Ground truth:** The standard `docker ps` header (and plan §3.5) uses three spaces between every column, including between `CREATED` and `STATUS`.
- **Impact:** Low — the block is explicitly labeled representative, but the header is described as "standard" and should match the real output spacing.
- **Fix:** Change line 94 to `CONTAINER ID   IMAGE   COMMAND   CREATED   STATUS   PORTS   NAMES` (three spaces between each column).

## Items verified as OK

- All 12 required sections are present in the specified order (§3.1–§3.12).
- Table of contents is present and links to every `##` section.
- No emojis, no HTML, no commented-out content, real newlines, language tags on all fenced blocks.
- All nine environment variables are listed in the correct order with the correct defaults (§2.1).
- The quoted `alpine-vm` JSONC block is byte-identical to the registration in `C:\Users\ibej_\.config\opencode\opencode.jsonc` (including `"enabled": false` and double-backslash path).
- Tool names, argument schema (`command` required string min 1, `timeoutMs` optional positive integer), and gate order match the source.
- All runtime message strings use the em dash (U+2014) and match the source / §2.2 (missing password, status messages, gate messages, rejection string, smoke-test output).
- Security notes cover plaintext-password rationale, prefix+space allowlist rule, disposable-VM context, no auto privileges, output hygiene, and SSH-key future hardening.
- Troubleshooting table includes all required symptoms plus the optional rejection and timeout rows.
- Project layout tree and per-file one-liners match §3.11.
- Verification record is dated 2026-10-04, tied to `8f602f1`, and cites only Task 2 evidence.
- Git state is correct: single `docs: add project readme` commit on `feat/mcp-server` directly atop `8f602f1`, worktree clean.

## Not in scope / not changed by this review

- No source files, `package.json`, `package-lock.json`, `.gitignore`, `opencode.jsonc`, TODO files, or branch state were modified.
- No new documentation files beyond this review/fix plan were created.

## Fix instructions summary

1. `README.md:36` — replace `. [truncated]` with `… [truncated]`.
2. `README.md:47` — remove the "First line, any state" table row.
3. `README.md:94` — correct `docker ps` header spacing to three spaces between every column.

After applying fixes, re-verify that no U+FFFD characters were introduced and that the file still exceeds 100 lines with all 12 sections in order.
