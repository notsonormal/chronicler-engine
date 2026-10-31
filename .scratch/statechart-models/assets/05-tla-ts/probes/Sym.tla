---- MODULE Sym ----
EXTENDS Naturals, FiniteSets, TLC
CONSTANTS Procs
VARIABLES turn, pc
vars == <<turn, pc>>
Init == turn \in Procs /\ pc = [p \in Procs |-> "idle"]
Next == \E p \in Procs : pc[p] = "idle" /\ pc' = [pc EXCEPT ![p] = "done"] /\ turn' = p
Spec == Init /\ [][Next]_vars
Sym == Permutations(Procs)
Inv == \A p \in Procs : pc[p] \in {"idle", "done"}
====
