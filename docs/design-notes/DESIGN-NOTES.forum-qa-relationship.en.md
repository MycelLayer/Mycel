# Forum and Q&A Relationship

Status: design draft

This note defines the relationship between the Forum App Layer and the Q&A App Layer.

## 0. Short Answer

The two notes are related, but they are not duplicates.

- the Forum App Layer is the more general discussion model
- the Q&A App Layer is a narrower resolution-oriented model
- both can exist conceptually
- implementation does not need to advance both at the same time

## 1. What the Forum App Layer Is Good At

The Forum App Layer is the broader discussion model.

Its center of gravity is:

- board structure
- thread structure
- reply trees
- explicit moderation history
- accepted reading for thread and board display

It is the better default model for:

- general discussion
- long-running threads
- moderator-visible disputes
- post-level visibility control
- branch-tolerant conversation systems

## 2. What the Q&A App Layer Is Good At

The Q&A App Layer is the narrower resolution model.

Its center of gravity is:

- one question
- multiple candidate answers
- one accepted answer under an active profile
- citation-oriented answer evaluation
- explicit answer-selection traces

It is the better model for:

- knowledge-base workflows
- answer selection
- expert-response systems
- citation-heavy answer review
- cases where one active result matters more than open-ended discussion

## 3. The Difference in Default Reading

The key difference is the unit of accepted reading.

For Forum:

- the accepted reading answers: "How should this thread or board be shown?"

For Q&A:

- the accepted reading answers: "Which answer is currently accepted for this question?"

This difference is enough to justify both notes.

## 4. Recommended Project Interpretation

The recommended interpretation is:

- Forum should be treated as the more general discussion app-layer example.
- Q&A should be treated as a specialized resolution-oriented app-layer example.
- Q&A may later be implemented as:
  - a specialized schema family beside Forum, or
  - a constrained profile built on top of the broader Forum shape.

This repository does not need to decide that implementation detail yet.

## 5. Recommended Near-term Choice

For near-term design and fixture work:

1. keep both notes
2. use Forum as the primary discussion example
3. defer Q&A schema or fixture work unless the accepted-answer model is specifically needed

This keeps the design space clear without forcing duplicate implementation tracks.
