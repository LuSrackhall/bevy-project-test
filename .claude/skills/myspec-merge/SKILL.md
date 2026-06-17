---
name: myspec-merge
description: "Sync with main, select merge method, merge, archive, and clean up. Handles the complete post-verification workflow including conflict resolution in the worktree."
---

# myspec-merge

Sync the worktree with the latest main branch, let the user choose a merge method, execute the merge, archive the change, and clean up the worktree.

**Input**: Optionally specify a change name. If omitted, check conversation context or prompt for selection.

## Steps

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context
   - Auto-select if only one active change exists
   - If ambiguous, run `openspec list --json` and use AskUserQuestion

   Announce: "Using change: <name>"

2. **Phase 1: Main sync check**

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

3. **Phase 1.5: Post-sync re-verification**

   After syncing main into the worktree (or if already in sync), re-verify the implementation:

   a. **Run myspec-verify skill** to re-check that the implementation still holds against the updated baseline. This includes:
   - Document verification (Completeness/Correctness/Coherence)
   - User acceptance (user must re-confirm after sync)
   - Iteration if issues are found

   b. **If verification or user acceptance fails:**
   > "Post-sync verification failed. Please fix issues before merging."
   > Return to myspec-apply or myspec-verify as needed. Do NOT proceed to merge.

   c. **If verification passes and user accepts:**
   > "Post-sync verification passed. Ready to merge."

4. **Phase 2: Merge method selection**

   Present three merge methods using AskUserQuestion:

   > "How would you like to merge change/<name> into main?"
   >
   > (Options are in English. The agent may translate to the user's preferred language if needed.)

   | Option | Description | Command |
   |--------|-------------|---------|
   | Create a merge commit | Preserves branch history with a merge commit | `git merge --no-ff change/<name>` |
   | Squash and merge | Compresses all commits into one commit on main | `git merge --squash change/<name>` + `git commit` |
   | Rebase | Replays commits on top of main for linear history | `git checkout main` + `git rebase change/<name>` |

4. **Phase 3: Execute merge**

   Based on the user's choice:

   **Merge commit:**
   ```bash
   cd <repo-root>
   git checkout main
   git merge --no-ff change/<name>
   ```

   **Squash and merge:**
   ```bash
   cd <repo-root>
   git checkout main
   git merge --squash change/<name>
   git commit
   ```
   (Use a conventional commit message summarizing the change.)

   **Rebase:**
   ```bash
   cd <repo-root>
   git checkout main
   git rebase change/<name>
   ```

   If merge conflicts occur, help resolve them. On resolution, continue with the chosen method.

   **IMPORTANT:** The user MUST confirm the merge. Never merge without asking.

5. **Phase 4: Archive**

   After a successful merge, run the openspec-archive-change skill:

   ```bash
   # The archive skill handles:
   # - Checking artifact completion
   # - Syncing delta specs to main specs
   # - Moving the change directory to archive/
   ```

   Commit the archive result:
   ```bash
   git add -A && git commit -m "archive: sync specs and archive change/<name>"
   ```

6. **Phase 5: Cleanup**

   Remove the worktree and delete the branch:

   ```bash
   git worktree remove .worktrees/change/<name>
   git branch -d change/<name>
   ```

   If the branch has unmerged changes (should not happen after merge), use `-D` only with user confirmation.

   Show final summary:
   ```
   ## Merge Complete

   **Change:** <name>
   **Method:** <merge commit / squash / rebase>
   **Archived to:** openspec/changes/archive/YYYY-MM-DD-<name>/
   **Worktree:** removed
   **Branch:** deleted
   ```

## Guardrails

- All main branch operations (pull, push, merge, rebase) MUST be confirmed by the user
- Worktree branch commits are handled automatically by the agent (conventional commit, user language/English)
- Resolve merge conflicts in the worktree when possible (during Phase 1 sync)
- Do NOT skip the merge method selection. The user MUST choose.
- Do NOT skip the archive step. It syncs delta specs to main specs.
- Do NOT skip the cleanup step. The worktree must be removed after merge.
- If any step fails, pause and report to the user. Do not proceed silently.
