# Research: the concepts in the driving adapters

Type: research
Status: open

## Question

Which concepts does the code in `src/adapters/driving/` (HTTP handlers, the inline Askama templates, view models and builders; about 4,800 lines without the unit tests) follow, and where do opposing or overlapping concepts meet in it?

Write the asset in the shape that the map's Notes give (Concepts, Meeting points, Shapes with no name, Questions). Do not rule on a concept or label a meeting point; write it as a question.

Dig the concepts out of the code. You get no list of concepts: do not open `assets/research-01-lens.md` or the other research assets. Read the code, and find the shapes that it repeats and the choices that it makes. Give each one the name that software-engineering books and well-known essays give it (a principle, an architecture style, a design style, a pattern or an idiom), and its opposite. When no such name fits, put the shape under Shapes with no name.

The HTMX fragment shape is in scope as it shows in the Rust code and the templates. The JavaScript and CSS files in `assets/` are out of scope.

## Done when

Every module in the scope is read, or is named in the asset as sampled with the reason. Every row cites `path:line` evidence and has a known, inferred or guessed label.
