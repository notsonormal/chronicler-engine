---- MODULE Temp ----
EXTENDS Naturals
VARIABLE x
vars == <<x>>
Init == x = 0
Next == x' = IF x < 3 THEN x + 1 ELSE x
FairSpec == Init /\ [][Next]_vars /\ WF_vars(Next)
Inv == x <= 3
Live == <>(x = 3)
====
