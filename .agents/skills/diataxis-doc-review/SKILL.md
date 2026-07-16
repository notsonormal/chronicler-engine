Read `chronicler_engine/docs-diataxis/explanation/diataxis.md` to gain an understanding of the Diataxis framework. 

When LLM writes documentation, it tend to write what it knows. Either from it's own knowledgebase or by reading the code. The problem is that this kind of documentation is useless, because it obvious stuff that it already knows and because reading code is a far better way to understand the code.

Getting an LLM to write good documentation is a struggle against the initial instincts of every LLM.

One of the purpose of the Diataxis framework more clearly organise the split between "understanding" (explaintion docs) and "information" (reference). However, start is just the standing point.

There are no use cases for tutorial and how-to documentation.

# Reference

When it comes to reference, the first instinct is to simply read the code and describe the code. But that is pointless because the LLM is going to read the code anything when it does work and the documentation is going to constantly become out of date.

So the real question is what value does each reference doc, and all of it's content, provide when the AI is going to read the code anyway and the code is reasonably self-documenting. 

Exactly what counts as 'good' reference docs and 'bad' reference docs hasn't been completely narrowed down yet.

# Explaination

In some way, explaination docs are an 'overflow' for refrences. We need to write more detail on certain topics, but don't want to bloat the 