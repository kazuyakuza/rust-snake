# Simplification Plan — VM SSH MCP Server · Task 1 (20261004, step 4.3)

Reviewer: code-simplifier. Contract: `.kilo/plans/20261004-vm-ssh-mcp-task1.md` + global plan
`.kilo/plans/20261004-vm-ssh-mcp.md`. Code reviewed at `C:\repo\vm-ssh-mcp` (branch
`feat/mcp-server`): 6 `src/*.mjs` modules + `scripts/smoke-test.mjs`.

No code was modified by this step. This file is the only deliverable.

## 0. Verdict

The implementation is already rule-compliant and closely follows the plan. Three small,
atomic, behavior-preserving simplifications are warranted (Sections 2–4). Everything else
was analyzed and deliberately left unchanged (Section 5). None of the proposals is a
medium/big redesign — the 50%-restricted implementer can execute all three verbatim, no
architector involvement required.

Rule baseline observed before proposing: every file ≤ 200 lines (max 152), every body
≤ 50 lines (max 25), ≤ 2 params everywhere, ≤ 2 nesting everywhere, no comments, exported
APIs match the plan exactly.

## 1. Execution rules for the implementer

- Apply steps S1, S2, S3 in order. Each step is one atomic edit pair (exact `before` →
  exact `after`). Do not improvise, do not "improve" anything beyond these three steps.
- Do not change any message string, any exported name, any env contract, any timeout,
  any JSON-RPC protocol element.
- After all three steps, run the verification block (Section 6) from
  `C:\repo\vm-ssh-mcp`. Any failure ⇒ stop and report; do not self-fix beyond the given
  snippets.

## 2. Step S1 — `src/vm-state-probe.mjs`: remove magic number `8` (the `"vmstate="` length)

Why: `parseVmStateToken` hardcodes both `"vmstate="` and `slice(8)`, and the two literals
are an implicit coupling (Code Guidelines #13 "avoid magic numbers"). A single named
constant keeps them consistent by construction. Behavior-preserving:
`"vmstate=".length === 8`, so the sliced offset is byte-identical; `startsWith` receives
the same string value.

Expected line delta: +2 (62 → 64 lines; still ≪ 200).

Edit 1/2 — add the constant. Before:

```js
const POWEROFF_STATE_TOKENS = ["poweroff", "saved", "aborted", "paused"];
```

After:

```js
const POWEROFF_STATE_TOKENS = ["poweroff", "saved", "aborted", "paused"];

const VM_STATE_LINE_PREFIX = "vmstate=";
```

Edit 2/2 — use it twice. Before:

```js
const parseVmStateToken = (output) => {
  const vmStateLine = output
    .split(/\r?\n/)
    .find((line) => line.toLowerCase().startsWith("vmstate="));
  if (vmStateLine === undefined) {
    return "";
  }
  return vmStateLine.slice(8).replaceAll('"', "").trim();
};
```

After:

```js
const parseVmStateToken = (output) => {
  const vmStateLine = output
    .split(/\r?\n/)
    .find((line) => line.toLowerCase().startsWith(VM_STATE_LINE_PREFIX));
  if (vmStateLine === undefined) {
    return "";
  }
  return vmStateLine.slice(VM_STATE_LINE_PREFIX.length).replaceAll('"', "").trim();
};
```

## 3. Step S2 — `src/ssh-command-runner.mjs`: collapse single-caller hop
`attachStreamHandlers` into `handleExecOpened`

Why: over-engineering pattern — `handleExecOpened` (single caller: the `exec` callback)
exists only to check `error` and then hand the same `context` to `attachStreamHandlers`
(also single caller). Two hops, one logical step ("exec opened → attach result handlers").
Merging removes the pure pass-through and one named function while keeping bodies ≤ 50
lines, nesting ≤ 2 (the `close` callback block sits at level 1, its statements at level 2;
the `if` block sits at level 1), and ≤ 2 params via the existing single context-object
parameter. Behavior-identical: same handler registration order, same
`connection.end()`-before-settle sequencing, same outcome shape. `startExec`'s call
`handleExecOpened({ ...context, error, stream })` needs no change — the spread still
carries `connection`, `settlers`, `config` (the unused `command` field is simply not
destructured).

Expected line delta: −5 (94 → 89 lines).

Edit — replace both functions. Before (lines 37–55 of the current file):

```js
const attachStreamHandlers = ({ stream, connection, settlers }) => {
  const buffers = { stdout: [], stderr: [] };
  stream.on("data", (chunk) => buffers.stdout.push(chunk));
  stream.on("stderr", (chunk) => buffers.stderr.push(chunk));
  stream.on("close", (code) => {
    connection.end();
    settlers.resolve(buildOutcome({ buffers, code }));
  });
};

const handleExecOpened = (context) => {
  const { error, stream } = context;
  if (error) {
    context.connection.end();
    context.settlers.reject(mapConnectionError(error, context.config));
    return;
  }
  attachStreamHandlers(context);
};
```

After:

```js
const handleExecOpened = ({ connection, settlers, config, error, stream }) => {
  if (error) {
    connection.end();
    settlers.reject(mapConnectionError(error, config));
    return;
  }
  const buffers = { stdout: [], stderr: [] };
  stream.on("data", (chunk) => buffers.stdout.push(chunk));
  stream.on("stderr", (chunk) => buffers.stderr.push(chunk));
  stream.on("close", (code) => {
    connection.end();
    settlers.resolve(buildOutcome({ buffers, code }));
  });
};
```

## 4. Step S3 — `src/server.mjs`: drop the two eta-wrapper closures in `registerTool`

Why: `async () => statusToolResult()` and `async (args) => runCommandToolResult(args)`
are redundant re-wrappings — they allocate a closure per registration that forwards every
argument unchanged. Passing the tool functions directly is the same call graph with one
fewer indirection. Behavior-preserving: the MCP SDK invokes the registered callback as
`callback(args, extra)`; both target functions ignore the extra parameter by design
(`vm_status` handler takes zero params, `vm_run_command` handler takes `args`), so direct
registration receives identical arguments and returns the identical result promises.
Note: the plan's Step 7 snippet showed the wrappers, but the pinned contract is the
behavior and the exported APIs (focus item 4 of the caller brief), not the illustrative
arrow syntax. Nothing observable to a client changes; the smoke test asserts this path.

Expected line delta: 0 (44 → 44 lines).

Edit 1/2. Before:

```js
    async () => statusToolResult(),
```

After:

```js
    statusToolResult,
```

Edit 2/2. Before:

```js
    async (args) => runCommandToolResult(args),
```

After:

```js
    runCommandToolResult,
```

## 5. Analyzed and intentionally NOT changed (with justification)

1. Cross-module duplication of `SSH not reachable on <host>:<port>`
   (`ssh-command-runner.mjs` thrown message vs `vm-tools.mjs` gate message): the strings
   differ (no trailing period vs trailing period) and unifying them would change
   user-visible text; a shared helper module would add a 7th `src/` file, deviating from
   the plan's target structure — structural, outside implementer authority. Defer: not
   worth the risk.
2. `isPositiveInteger` (`vm-connection-config.mjs`): inlining it into
   `parsePositiveInteger` would create a multi-section `if` condition, violating the
   single-section boolean rule. Keep.
3. `command-policy.mjs`: verbatim plan Step 3, 8 lines, `&&` lives in a return position
   and is the plan's own documented single-section gate. Keep as-is; any edit is contract
   deviation.
4. `vm-tools.mjs`: `textResult` is the shared helper the plan mandates; the near-duplicate
   status/gate messages across `buildStatusReport` and `findGateFailure` are plan-pinned
   exact strings — collapsing them changes behavior. No safe simplification found; keep.
5. `smoke-test.mjs` `completeWhenStatusReceived`: looks like a single-caller over-abstraction,
   but inlining the `if (results.has(3))` into the `stdout` data listener would put its
   body at nesting level 3 (promise executor → data callback → if). The extraction is
   required by the ≤ 2 rule. Keep. Same reasoning keeps the two early-return guards in
   `recordResponse` (a merged `||`/`&&` guard is multi-section).
6. `resolveExitCode`, `rejectConnection`, `startExec` (`ssh-command-runner.mjs`): each has
   one caller, but each names a non-obvious step (null-exit-code default, end+reject pair,
   exec kickoff) and inlining either raises density or risks the nesting cap
   (`startExec` inside the `ready` listener would produce a 3-deep chained callback
   expression). Value of the names ≥ cost of the hops. Keep (only the clear
   pass-through pair from S2 is collapsed).
7. Timeout guards (`settlers` in runner vs `guard` in smoke-test): same idea, different
   modules, one is `src` and the other `scripts/`; plan pins both protocols individually.
   No legal shared home exists. Keep.
8. `probeSsh` / `isCleanEcho`, `truncateOutput`, `executeAllowedCommand` shallow-spread
   derived config: verbatim plan behavior with correct helper splits. Keep.

Conclusion: Task 1 code does NOT need a large-scale simplification; the three atomic steps
above are the complete warranted scope. Nothing needs deferral to a new TODO.

## 6. Verification block (implementer runs after S1–S3, from `C:\repo\vm-ssh-mcp`)

1. Syntax check the three touched modules (offline, VM-independent):

```text
node --check src/vm-state-probe.mjs
node --check src/ssh-command-runner.mjs
node --check src/server.mjs
```

2. Offline regression (behavior gate for this plan): `npm run smoke-test` — must exit 0 and
   print `SMOKE TEST PASS` (VM may be OFF: `vm_status` line mentioning `not running` or
   `Could not determine` is a PASS per the script's pattern).
3. Rule re-check of the three files: `attachStreamHandlers` no longer exists
   (`node --check` success implies nothing; confirm via search), `handleExecOpened` body
   ≤ 50 lines, all files still ≤ 200 lines.
4. Full re-verification is owned by Task 2 — do not run live VM commands here.

## 7. Hand-off

- Commit suggestion for the implementer of step 4.6 (on `feat/mcp-server`, after passing
  Section 6, single commit): `refactor: simplify vm state parsing, exec handling and tool registration`.
- Step 4.5b architector-adherence: S3 deviates from the plan's illustrative arrow syntax
  in Step 7 while preserving the pinned behavior/API — flag this paragraph if the checker
  does literal-text comparisons.
