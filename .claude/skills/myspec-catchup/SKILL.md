---
name: myspec-catchup
description: "Sync worktree with latest main and re-verify. Use to catch up with main changes before merging, or as a standalone check."
---

# myspec-catchup

Sync the worktree branch with the latest main, then re-verify that the implementation still works. Can be used standalone or called by myspec-merge.

**Input**: Optionally specify a change name. If omitted, check conversation context or prompt for selection.

## Steps

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context
   - Auto-select if only one active change exists
   - If ambiguous, run `openspec list --json` and use AskUserQuestion

   Announce: "Using change: <name>"

2. **Main sync check**

   Check if local main and origin/main are in sync:

   ```bash
   git fetch origin
   LOCAL_MAIN=$(git rev-parse main)
   ORIGIN_MAIN=$(git rev-parse origin/main)
   ```

   **If origin/main is ahead of local main:**
   > "origin/main has N new commit(s) that local main does not have. Should I pull to update local main?"

   Use AskUserQuestion. If user confirms:
   ```bash
   git checkout main
   git pull origin main
   git checkout change/<name>
   git merge main
   ```
   If conflicts arise during `git merge main`, resolve them in the worktree. Report conflicts to the user and assist with resolution.

   **If local main is ahead of origin/main:**
   > "Local main has N new commit(s) not pushed to origin. Should I push to origin?"

   Use AskUserQuestion. If user confirms:
   ```bash
   git checkout main
   git push origin main
   git checkout change/<name>
   ```

   **If in sync:**
   > "Local main and origin/main are in sync. Skipping sync step."

   **IMPORTANT:** All main branch operations (pull, push) MUST be confirmed by the user. Never execute main branch operations without explicit user approval.

3. **Post-sync re-verification**

   After syncing main into the worktree (or if already in sync), re-verify the implementation:

   a. **Run myspec-verify skill** to re-check that the implementation still holds against the updated baseline. This includes:
   - Document verification (Completeness/Correctness/Coherence)
   - User acceptance (user must re-confirm after sync)
   - Iteration if issues are found

   b. **If verification or user acceptance fails:**
   > "Post-sync verification failed. Please fix issues before continuing."
   > Return to myspec-apply or myspec-verify as needed.

   c. **If verification passes and user accepts:**
   > "Catchup complete. Worktree is up to date with main and verified."

## Guardrails

- All main branch operations (pull, push) MUST be confirmed by the user
- Resolve merge conflicts in the worktree when possible
- Do NOT skip re-verification after sync — syncing may introduce issues
- If called standalone (not from myspec-merge), just report completion. Do NOT proceed to merge automatically.
