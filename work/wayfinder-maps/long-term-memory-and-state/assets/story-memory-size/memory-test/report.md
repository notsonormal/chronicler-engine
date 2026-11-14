## Shape `full`

Kept questions (no-memory control wrong in every run): 35 of 42

| Arm | Runs | Kept: fact | Kept: time | Kept: who | Kept: all, mean (min-max) | All questions | Calls per run | Prompt tokens per run | Memory words |
|---|---|---|---|---|---|---|---|---|---|
| none | 3 | 0/21 / 0/21 / 0/21 | 0/10 / 0/10 / 0/10 | 0/4 / 0/4 / 0/4 | 0 % (0-0) | 6/42 / 7/42 / 7/42 | 45 | 642,272 | 0 |
| whole | 3 | 21/21 / 21/21 / 21/21 | 10/10 / 9/10 / 9/10 | 3/4 / 3/4 / 3/4 | 95 % (94-97) | 41/42 / 40/42 / 40/42 | 45 | 1,205,597 | 0 |
| base_1000_shorten | 3 | 16/21 / 16/21 / 16/21 | 6/10 / 6/10 / 6/10 | 3/4 / 3/4 / 3/4 | 71 % (71-71) | 32/42 / 32/42 / 32/42 | 47 | 689,555 | 1145 |
| base_rolling_summary | 3 | 16/21 / 16/21 / 15/21 | 9/10 / 9/10 / 10/10 | 2/4 / 2/4 / 2/4 | 77 % (77-77) | 34/42 / 34/42 / 34/42 | 51 | 670,906 | 860 |
| base_rolling_summary_v1 | 3 | 15/21 / 18/21 / 14/21 | 8/10 / 7/10 / 7/10 | 1/4 / 2/4 / 2/4 | 70 % (66-77) | 31/42 / 34/42 / 30/42 | 48 | 660,573 | 709 |
| one_summary | 3 | 12/21 / 15/21 / 11/21 | 9/10 / 10/10 / 7/10 | 2/4 / 2/4 / 2/4 | 67 % (57-77) | 30/42 / 34/42 / 27/42 | 50 | 732,758 | 503 |

## Shape `short`

Kept questions (no-memory control wrong in every run): 12 of 42

| Arm | Runs | Kept: fact | Kept: time | Kept: who | Kept: all, mean (min-max) | All questions | Calls per run | Prompt tokens per run | Memory words |
|---|---|---|---|---|---|---|---|---|---|
| none | 1 | 0/6 | 0/4 | 0/2 | 0 % (0-0) | 0/12 | 12 | 1,296 | 0 |
| whole | 1 | 6/6 | 4/4 | 1/2 | 92 % (92-92) | 11/12 | 12 | 301,295 | 0 |
| one_summary | 1 | 3/6 | 3/4 | 1/2 | 58 % (58-58) | 7/12 | 12 | 9,340 | 490 |

