# RFC 0000: Feature Name Here

- **Start Date:** YYYY-MM-DD
- **RFC PR:** [#0000](https://github.com/velalang/vela-rfcs/pull/0000)
- **Tracking Issue:** [#0000](https://github.com/velalang/vela/issues/0000)
- **Author:** @your-github-username

---

## Summary

> One paragraph explanation of the feature.

Brief, high-level overview of what you're proposing. This should be understandable by someone who isn't familiar with the technical details.

**Example:**
> This RFC proposes adding pattern matching to Vela, allowing developers to match values against patterns and destructure data structures in a concise, type-safe manner.

---

## Motivation

> Why are we doing this? What use cases does it support? What problems does it solve? What is the expected outcome?

### Problem Statement

Describe the current pain points or limitations that motivate this RFC.

**Example:**
```vela
// Current approach is verbose
result: Result<UserData, Error> = getUserData()
if result.type === "success" {
  data: UserData = result.value
  // handle success
} else if result.type === "error" {
  error: Error = result.error
  // handle error
}
```

### Proposed Solution

High-level overview of your solution.

**Example:**
```vela
// Proposed pattern matching syntax
match getUserData() {
  Success(data) => // handle success,
  Error(err) => // handle error,
}
```

### Use Cases

List concrete scenarios where this feature would be beneficial.

1. **Use Case 1:** Description
2. **Use Case 2:** Description
3. **Use Case 3:** Description

---

## Detailed Design

> This is the bulk of the RFC. Explain the design in enough detail for somebody familiar with Vela to understand and implement.

### Syntax

Define the exact syntax for the feature.

**Grammar (EBNF):**
```
match_expression = "match" expression "{" match_arm+ "}" ;
match_arm        = pattern "=>" expression "," ;
pattern          = identifier | literal | constructor_pattern ;
```

**Example Code:**
```vela
match value {
  Some(x) => x,
  None => 0,
}
```

### Semantics

Explain the runtime behavior.

- **Type checking:** How types are validated
- **Evaluation order:** Left-to-right, eager vs lazy
- **Pattern matching:** Exhaustiveness checking
- **Memory model:** Stack vs heap, ownership

### Type System Integration

How does this interact with Vela's type system?

```vela
type Result<T, E> = Success(T) | Error(E);

fn processResult<T, E>(r: Result<T, E>) -> T {
  match r {
    Success(value) => value,
    Error(err) => panic("Error: {err}"),
  }
}
```

### Error Handling

How are errors reported?

**Compile-time errors:**
- Non-exhaustive match
- Unreachable patterns
- Type mismatches

**Runtime errors:**
- None (exhaustiveness guarantees safety)

### Edge Cases

Address unusual scenarios.

1. **Empty matches:** Not allowed (compile error)
2. **Overlapping patterns:** Last pattern wins (with warning)
3. **Pattern guards:** Supported (future extension)

### Examples

Provide comprehensive examples.

#### Example 1: Basic Pattern Matching

```vela
fn describe(x: i32) -> String {
  match x {
    0 => "zero",
    1 => "one",
    _ => "many",
  }
}
```

#### Example 2: Destructuring

```vela
type Point = Point(x: f64, y: f64);

fn distanceFromOrigin(p: Point) -> f64 {
  match p {
    Point(x, y) => sqrt(x * x + y * y),
  }
}
```

#### Example 3: Nested Patterns

```vela
match result {
  Success(Some(value)) => print(value),
  Success(None) => print("No value"),
  Error(err) => print("Error: {err}"),
}
```

---

## Rationale and Alternatives

> Why is this design the best in the space of possible designs?

### Design Decisions

#### Decision 1: Exhaustiveness Checking

**Choice:** Enforce exhaustiveness at compile time  
**Rationale:** Prevents runtime errors, aligns with Vela's safety goals  
**Trade-off:** Requires more upfront work from developers

#### Decision 2: Pattern Syntax

**Choice:** `Pattern(field1, field2)` instead of `Pattern { field1, field2 }`  
**Rationale:** More concise, familiar to functional programmers  
**Trade-off:** Less explicit than struct-style syntax

### Alternatives Considered

#### Alternative 1: No Pattern Matching

**Description:** Keep using `if`/`else` chains  
**Pros:** No language complexity added  
**Cons:** Verbose, error-prone, doesn't scale

**Why rejected:** Pattern matching is a core feature of modern languages, essential for ergonomic error handling and data destructuring.

#### Alternative 2: Method-Based Matching

**Description:** Use methods like `value.match({ Success: fn, Error: fn })`  
**Pros:** No new syntax  
**Cons:** Less performant, awkward type checking, non-idiomatic

**Why rejected:** Doesn't integrate well with type system, poor ergonomics.

### Prior Art

Research from other languages:

- **Rust:** Exhaustive match, `_` wildcard
- **Swift:** `switch` statement with pattern matching
- **OCaml:** Pattern matching is core language feature
- **Scala:** `match` with sealed traits

**Lessons learned:** Exhaustiveness checking is critical, wildcards are useful, pattern guards add flexibility.

### Impact on Existing Code

- **Breaking changes:** None (new feature)
- **Deprecations:** None
- **Migration path:** N/A (additive change)

---

## Unresolved Questions

> What parts of the design do you expect to resolve through the RFC process before this gets merged?

1. **Pattern guards:** Should we support `match x { Some(y) if y > 10 => ... }` in v1?
2. **Irrefutable patterns:** Should pattern destructuring be allowed (NO let keyword in Vela)?
3. **Performance:** Can we guarantee zero-cost abstraction for simple matches?

---

## Future Possibilities

> Think about what the natural extension of your proposal would be and how it would affect the project as a whole in a holistic way. This is also a good place to "dump ideas", if they are out of scope for the RFC but are otherwise related.

### Short-term Extensions

- **Pattern guards:** `match x { Some(y) if y > 10 => ... }`
- **Multiple patterns per arm:** `match x { 1 | 2 | 3 => ... }`
- **Range patterns:** `match x { 0..10 => ... }`

### Long-term Possibilities

- **Active patterns:** Custom pattern extractors
- **View patterns:** Pattern matching with transformations
- **Or-patterns in bindings:** `(Some(x) | None) = value` (immutable by default)

### Interaction with Future Features

- **Algebraic effects:** Pattern match on effect handlers
- **Type classes:** Pattern match on trait implementations
- **Macros:** Generate match arms programmatically

---

## Appendix A: Performance Considerations

(Optional section for performance-critical features)

### Compilation Strategy

- **Decision tree generation:** O(n) patterns
- **Code generation:** Jump tables for enums
- **Optimization:** Dead code elimination for unreachable arms

### Benchmarks

(Include benchmark results if available)

---

## Appendix B: Implementation Plan

(Optional section outlining implementation phases)

### Phase 1: Core Feature (v0.1)

- [ ] Parser support for `match` syntax
- [ ] Type checker integration
- [ ] Exhaustiveness checking
- [ ] Code generation

### Phase 2: Improvements (v0.2)

- [ ] Pattern guards
- [ ] Range patterns
- [ ] Performance optimizations

### Phase 3: Advanced Features (v1.0)

- [ ] Active patterns
- [ ] Or-patterns in bindings

---

## Acknowledgments

Thank contributors who helped shape this RFC.

- @contributor1 for suggesting pattern guards
- @contributor2 for exhaustiveness algorithm
- Community members in [Discussion #123](https://github.com/velalang/vela/discussions/123)

---

*This template is based on [Rust RFC Template](https://github.com/rust-lang/rfcs/blob/master/0000-template.md)*
