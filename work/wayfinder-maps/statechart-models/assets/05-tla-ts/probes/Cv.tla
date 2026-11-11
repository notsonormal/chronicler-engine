---- MODULE Cv ----
EXTENDS Naturals
VARIABLE n
vars == <<n>>
Init == n = 0
Next == n' = n + 1
View == n % 2
Bound == n < 6
Inv == n >= 0
====
