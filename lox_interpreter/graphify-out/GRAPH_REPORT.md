# Graph Report - lox_interpreter  (2026-05-25)

## Corpus Check
- 4 files · ~6,867 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 60 nodes · 175 edges · 6 communities detected
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 9|Community 9]]

## God Nodes (most connected - your core abstractions)
1. `Parser` - 30 edges
2. `Interpreter` - 10 edges
3. `Environment` - 3 edges
4. `Scanner` - 2 edges
5. `LoxValue` - 2 edges
6. `ParserError` - 1 edges
7. `TokenTypes` - 1 edges
8. `Token` - 1 edges
9. `Expr` - 1 edges
10. `Stmt` - 1 edges

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities (11 total, 5 thin omitted)

### Community 2 - "Community 2"
Cohesion: 0.33
Nodes (5): Expr, ParserError, Stmt, Token, TokenTypes

## Knowledge Gaps
- **7 isolated node(s):** `ParserError`, `TokenTypes`, `Token`, `Expr`, `Stmt` (+2 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Parser` connect `Community 1` to `Community 0`, `Community 8`, `Community 2`, `Community 4`?**
  _High betweenness centrality (0.173) - this node is a cross-community bridge._
- **Why does `Interpreter` connect `Community 3` to `Community 6`, `Community 7`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._
- **What connects `ParserError`, `TokenTypes`, `Token` to the rest of the system?**
  _7 weakly-connected nodes found - possible documentation gaps or missing edges._