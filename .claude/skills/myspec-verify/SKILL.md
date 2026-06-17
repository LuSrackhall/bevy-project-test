---
name: myspec-verify
description: "Verify implementation, get user acceptance, and handle iteration. Wraps openspec-verify-change with user acceptance checkpoint and iteration decision loop."
---

# myspec-verify

Verify implementation against change artifacts, present results to the user for acceptance, and handle iteration if the user is not satisfied.

**Input**: Optionally specify a change name. If omitted, check conversation context or prompt for selection.

## Steps

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context
   - Auto-select if only one active change exists
   - If ambiguous, run `openspec list --json` and use AskUserQuestion

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

4. **Phase 2: User acceptance**

   Present a change summary to the user:

   ```
   ## Verification Summary

   **Change:** <name>

   | Dimension | Status |
   |-----------|--------|
   | Completeness | X/Y tasks, N reqs covered |
   | Correctness | M/N reqs implemented |
   | Coherence | Issues found / Clean |

   ### Key Changes
   - <file>: <what changed>
   - ...

   ### Issues (if any)
   - CRITICAL: ...
   - WARNING: ...
   ```

   Then ask: **"Does this change meet your requirements?"**

5. **Phase 3a: User accepts**

   Backfill ALL artifacts to match the final implementation:
   - Update brainstorm-spec.md if design diverged
   - Update proposal.md if scope changed
   - Update specs/ if requirements were adjusted
   - Update design.md if implementation approach changed
   - Update tasks.md if tasks were added/removed/modified

   Commit the backfilled artifacts:
   ```bash
   git add -A && git commit -m "docs: backfill artifacts to match implementation"
   ```

   Then prompt: **"Artifacts updated. Run myspec-merge skill to sync with main, merge, and archive."**

6. **Phase 3b: User does not accept**

   a. **Analyze the root cause:**
   - What went wrong?
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

- Do NOT skip the user acceptance step. The user MUST explicitly confirm.
- Do NOT proceed to merge or archive. Those are handled by myspec-merge.
- Do NOT run build or test. Those are the user's responsibility.
- When backfilling artifacts, update ALL artifacts, not just the ones that drifted.
- When recommending iteration strategies, always lead with the recommended one and explain why.
