---
name: myspec-verify
description: Verify implementation against change artifacts using machine acceptance (tests, determinism, golden hash, replay round-trip, AI self-play), then handle iteration. Human input is requested only for aspects machines cannot judge.
---

# myspec-verify

Verify implementation against change artifacts, **run the machine acceptance gate yourself**, record the verdict in `verify.md`, and handle iteration when the gate fails (or when a human raises an objection).

**Input**: Optionally specify a change name. If omitted, check conversation context or prompt for selection.

## Steps

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context
   - Auto-select if only one active change exists
   - If ambiguous, run `openspec list --json` and use ask_user_question

   Announce: "Using change: <name>"

2. **Get context files**

   ```bash
   openspec instructions apply --change "<name>" --json
   ```

   Read all files from `contextFiles` (brainstorm-spec, proposal, specs, design, tasks).

3. **Phase 1: Document verification**

   Perform three-dimensional verification:

   **Completeness:**
   - Check all tasks.md checkboxes: `- [x]` vs `- [ ]`
   - Check delta spec requirements against codebase for coverage

   **Correctness:**
   - Map each requirement to implementation evidence in code
   - Check scenario coverage

   **Coherence:**
   - Verify implementation follows design.md decisions
   - Check code pattern consistency

   Record findings as CRITICAL / WARNING / SUGGESTION.

4. **Phase 2: Machine acceptance (you run it, not the user)**

   **You are responsible for running the build and the tests.** Never hand this step to the user.

   ```bash
   # 1) Static gates
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings

   # 2) Simulation + integration tests
   cargo test -p simulation
   WGPU_BACKEND=noop cargo test --workspace

   # 3) Constitution guards
   python3 scripts/check-hash-coverage.py          # §10.2 hash coverage

   # 4) Determinism + replay round-trip (§10.1 / §10.2)
   cargo run -p sim-cli -- scenario --seed 42 --map small --ticks 500 \
     --repeat 2 --record /tmp/verify-scenario.ron --json

   # 5) If the change touches simulation behaviour: a match must still be decided
   cargo run -p sim-cli -- selfplay --seed 42 --map small --ticks 4000 \
     --symmetric --require-decided
   ```

   **Pass criteria (all must hold):**

   - Every command above exits `0` (`sim-cli`: `0` pass / `1` verification failed / `2` usage or format error)
   - `sim-cli scenario --json` reports `determinism.stable == true`
   - The record→replay comparison reports `replay.mismatches == 0`
   - If the change is **not** supposed to change behaviour (refactor / docs / config),
     the golden hash must equal the pre-change value. Any difference is a regression.
   - If the change **is** supposed to change behaviour, record the old → new golden hash
     (`sim-cli scenario … --quiet` prints it) and state explicitly that it is expected.

   Record the verdict in `verify.md`: each command, its exit code, the key JSON fields, and the conclusion.

   Then present a short summary (no question needed unless Phase 2b applies):

   ```
   ## Machine Verification Summary

   **Change:** <name>

   | Gate | Result |
   |------|--------|
   | fmt / clippy | pass/fail |
   | test -p simulation | N passed |
   | test --workspace | N passed |
   | hash coverage guard | pass/fail |
   | determinism + replay round-trip | pass/fail (mismatches=0) |
   | AI self-play decided | pass/fail (winner=faction N) |
   | golden hash | unchanged / old → new (expected) |

   ### Key Changes
   - <file>: <what changed>
   ```

5. **Phase 2b: Human input (only for what machines cannot judge)**

   Ask the user **only** when the change has aspects no automated gate can decide —
   visual appearance, game feel, scope or priority trade-offs.

   Keep it explicit and non-blocking:

   ```
   机器验收已通过（见上表）。
   以下属于机器无法判定的部分，需要你的意见：<列出具体点>
   你可以直接回复，也可以让我先继续合并——异议会走回退流程。
   ```

   Do NOT ask the user to run the build/tests, and do NOT block on a reply when the
   change has no machine-unjudgeable aspect.

6. **Phase 3a: Gate passes**

   Backfill ALL artifacts to match the final implementation. Do NOT skip any artifact.

   For EACH artifact, read it, compare against the actual implementation, and update:

   1. **brainstorm-spec.md** — update Context/Decisions/Risks to match what was actually built
   2. **proposal.md** — update What Changes/Capabilities/Impact to match actual scope
   3. **specs/** — update each delta spec to reflect actual requirements implemented
   4. **design.md** — update Decisions to match actual implementation approach
   5. **tasks.md** — update task list to match all tasks actually completed (add missing, remove unused)

   **IMPORTANT:** You MUST check EVERY artifact, not just the ones you think changed.
   Implementation often diverges from the original plan in ways that affect multiple artifacts.

   After updating, verify completeness:
   - List all artifacts and confirm each was reviewed and updated
   - If any artifact was not touched, review it again

   Commit the backfilled artifacts (include `verify.md`):

   ```bash
   git add -A && git commit -m "docs: backfill artifacts to match implementation"
   ```

   Then prompt: **"Artifacts updated. Run myspec-merge skill to sync with main, merge, and archive."**

7. **Phase 3b: Gate fails, or a human objects**

   a. **Analyze the root cause:**
   - Which gate failed, and what does its output say?
   - Is it a minor implementation issue or a fundamental approach problem?

   b. **Recommend an iteration strategy:**

   | Strategy | When to recommend |
   |----------|------------------|
   | Fix in place | Implementation detail issues, edge cases (default) |
   | New change in same worktree | Need to re-plan, existing code is useful reference |
   | Git reset + stash reference | Need clean baseline but want to keep code as reference |
   | Git reset, full redo | Fundamental approach error |
   | Abandon change | Requirements need redefining |

   Present recommendation with reasoning.

   c. **Let the user choose** (they may pick a different strategy).

   d. **Execute the chosen strategy:**
   - Fix in place → return to myspec-apply skill
   - New change → `openspec new change "<new-name>"`, keep old code
   - Git reset + stash → `git stash && git reset --hard <pre-impl-commit>`
   - Git reset → `git reset --hard <pre-impl-commit>`
   - Abandon → prompt user to run cleanup manually

   e. After executing strategy, prompt: **"Run myspec-apply skill to re-implement."**

## Guardrails

- **You MUST run the build, the tests, and the `sim-cli` gates yourself.** They are the agent's
  responsibility, not the user's. (This replaces the earlier rule that forbade the agent from
  running them.)
- The machine gate is authoritative: do not merge while any gate above fails.
- Do NOT block on the user for changes without machine-unjudgeable aspects; ask only about
  appearance / feel / scope, and treat it as non-blocking.
- Never silently accept a golden-hash change: state old → new and whether it is expected.
- Do NOT proceed to merge or archive. Those are handled by myspec-merge.
- When backfilling artifacts, update ALL artifacts, not just the ones that drifted.
- When recommending iteration strategies, always lead with the recommended one and explain why.
