# 05 - Quick Routing

Use this decision tree to find the minimum docs you need for the current task.

## Decision tree

1. **Does the task touch `vendor/` code?**
   - YES → Read `10-vendor-policy.md`, then `20-vendor-llmdocs-workflow.md`. Stop.
   - NO → Continue.

2. **Does the task involve build, test, or git operations?**
   - YES → Read `30-dev-env-and-workflow.md`. Stop.
   - NO → Continue.

3. **Is this a general orientation / "what is this repo?" question?**
   - YES → Read `00-scope.md`. Stop.
   - NO → Skim `00-scope.md` for context, then proceed with the task.
