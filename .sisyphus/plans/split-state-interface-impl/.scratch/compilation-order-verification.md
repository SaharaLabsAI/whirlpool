VERDICT: PASS (0 ordering violations)

## Checks
1. Task ordering follows migration sequence 01 -> 06.
2. No task requires outputs from a later task.
3. Interface/type stabilization (Task 01) precedes implementation movement (Task 03).
4. Consumer rewires (Tasks 04-05) occur after concrete exports are established (Task 03).
5. Cleanup/removal task (Task 06) comes after all consumer rewires.
