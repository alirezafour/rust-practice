# Graph Report - lox_interpreter  (2026-05-31)

## Corpus Check
- 4 files · ~11,632 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 184 nodes · 477 edges · 8 communities detected
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]

## God Nodes (most connected - your core abstractions)
1. `assert_program_ok()` - 32 edges
2. `Parser` - 31 edges
3. `assert_runtime_error()` - 17 edges
4. `parse_program_from()` - 10 edges
5. `assert_parse_error()` - 10 edges
6. `Interpreter` - 10 edges
7. `assert_scan_error()` - 8 edges
8. `lit_num()` - 8 edges
9. `mk_token()` - 7 edges
10. `lit_id()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `assert_program_ok()` --calls--> `run_program()`  [EXTRACTED]
  interpreter.rs → interpreter.rs  _Bridges community 4 → community 3_

## Communities (8 total, 1 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.2
Nodes (11): parse_block(), parse_function(), parse_if(), parse_if_else(), parse_print(), parse_return_nil(), parse_return_value(), parse_var_nil() (+3 more)

### Community 1 - "Community 1"
Cohesion: 0.13
Nodes (36): assert_parse_error(), Expr, exprs_assign(), exprs_binary(), lit_false(), lit_id(), lit_nil(), lit_num() (+28 more)

### Community 2 - "Community 2"
Cohesion: 0.1
Nodes (29): assert_scan_error(), Expr, scan_error_invalid_escape(), scan_error_number_then_letter(), scan_error_unexpected_newline_in_escape(), scan_error_unknown_char(), scan_error_unterminated_string(), scan_error_unterminated_string_multiline() (+21 more)

### Community 3 - "Community 3"
Cohesion: 0.12
Nodes (32): assert_program_ok(), class_creation(), class_empty(), class_method_with_params(), class_multiple_instances(), class_multiple_methods(), class_object(), class_object_function_call() (+24 more)

### Community 4 - "Community 4"
Cohesion: 0.22
Nodes (5): Environment, inter_fun_capture(), inter_var(), Interpreter, run_program()

### Community 5 - "Community 5"
Cohesion: 0.13
Nodes (15): assert_runtime_error(), runtime_error_add_string_number(), runtime_error_arithmetic_non_number(), runtime_error_assign_undefined(), runtime_error_call_field(), runtime_error_call_non_function(), runtime_error_compare_string(), runtime_error_divide_by_zero() (+7 more)

### Community 6 - "Community 6"
Cohesion: 0.83
Nodes (3): get_statements(), looped(), main()

## Knowledge Gaps
- **8 isolated node(s):** `TokenTypes`, `ScannerError`, `ParserError`, `RuntimeError`, `TokenTypes` (+3 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Parser` connect `Community 0` to `Community 1`?**
  _High betweenness centrality (0.037) - this node is a cross-community bridge._
- **Why does `Interpreter` connect `Community 4` to `Community 3`?**
  _High betweenness centrality (0.023) - this node is a cross-community bridge._
- **Why does `assert_program_ok()` connect `Community 3` to `Community 4`?**
  _High betweenness centrality (0.015) - this node is a cross-community bridge._
- **What connects `TokenTypes`, `ScannerError`, `ParserError` to the rest of the system?**
  _8 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.13 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.1 - nodes in this community are weakly interconnected._
- **Should `Community 3` be split into smaller, more focused modules?**
  _Cohesion score 0.12 - nodes in this community are weakly interconnected._