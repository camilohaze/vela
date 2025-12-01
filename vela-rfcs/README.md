# Vela RFCs

This repository contains **Requests for Comments (RFCs)** for substantial changes to the Vela programming language.

---

## What is an RFC?

An RFC is a design document that describes a new feature, improvement, or significant change to Vela. It provides:

- 📋 **Detailed specification** of the proposed change
- 🎯 **Motivation** and use cases
- 🔍 **Analysis** of alternatives and trade-offs
- 💬 **Community discussion** before implementation

---

## When to Write an RFC

**✅ RFC Required:**
- New language syntax or semantics
- Breaking changes to stable APIs
- Major architectural changes
- New standard library modules
- Changes to the type system
- Tooling architecture changes

**❌ RFC Not Required:**
- Bug fixes
- Documentation improvements
- Performance optimizations (without API changes)
- Internal refactoring
- Test additions

**💡 Not Sure?** Ask in [GitHub Discussions](https://github.com/velalang/vela/discussions)!

---

## RFC Process

### Lifecycle

```
1. Draft       →  Author writes initial proposal
2. Discussion  →  Community provides feedback
3. Review      →  Core Team evaluates
4. Decision    →  Accepted or Rejected
5. Implement   →  Feature development begins
```

### Timeline

- **Draft:** As long as needed
- **Discussion:** 2-4 weeks minimum
- **Review:** 1-2 weeks
- **Implementation:** After acceptance

---

## How to Submit an RFC

### Step 1: Fork and Clone

```bash
git clone https://github.com/velalang/vela-rfcs.git
cd vela-rfcs
```

### Step 2: Copy the Template

```bash
cp 0000-template.md text/0000-my-feature.md
```

### Step 3: Write Your RFC

Fill in the template with your proposal. See [Template Guide](#template-guide) below.

### Step 4: Submit Pull Request

```bash
git add text/0000-my-feature.md
git commit -m "RFC: My Feature Name"
git push origin my-feature-branch
```

Create a PR with:
- **Title:** `RFC: My Feature Name`
- **Description:** Brief summary and links

### Step 5: Discussion

- Respond to feedback in PR comments
- Update RFC based on discussion
- Iterate until consensus emerges

### Step 6: Core Team Review

Technical Leads will:
1. Review technical soundness
2. Check alignment with project goals
3. Consider implementation complexity
4. Make final decision

### Step 7: Decision

**Accepted:**
- RFC merged
- Assigned RFC number
- Implementation can begin

**Rejected:**
- RFC closed with detailed rationale
- Can be revised and resubmitted

---

## Template Guide

### Required Sections

#### 1. Summary

One-paragraph explanation of the feature.

**Example:**
> This RFC proposes adding pattern matching to Vela, allowing developers to match values against patterns and destructure data in a concise, type-safe way.

#### 2. Motivation

Why are we doing this? What problems does it solve?

**Include:**
- Real-world use cases
- Pain points with current approach
- Benefits to developers

#### 3. Detailed Design

Technical specification of the change.

**Include:**
- Syntax (if applicable)
- Semantics
- Examples
- Edge cases
- Error handling

#### 4. Rationale and Alternatives

Why this design over others?

**Include:**
- Design decisions and trade-offs
- Alternative approaches considered
- Why alternatives were not chosen
- Impact on existing code

#### 5. Unresolved Questions

What aspects need further discussion?

**Examples:**
- Naming bikeshed
- Implementation details
- Interaction with future features

#### 6. Future Possibilities

What could we build on this later?

**Examples:**
- Extensions to the feature
- Related features
- Long-term vision

---

## RFC Numbering

RFCs are numbered sequentially:

- `0000-template.md` - Template (not an RFC)
- `0001-reactive-signals.md` - First RFC
- `0002-pattern-matching.md` - Second RFC
- etc.

Numbers are assigned **after acceptance** by the Core Team.

---

## RFC Status

| Status | Meaning |
|--------|---------|
| 🟡 **Draft** | Author is still working on it |
| 💬 **Discussion** | Open for community feedback |
| 🔍 **Core Review** | Under Technical Lead review |
| ✅ **Accepted** | Approved for implementation |
| ❌ **Rejected** | Not accepted (with rationale) |
| 🚧 **Implementing** | Feature is being built |
| ✔️ **Implemented** | Shipped in a release |
| 🗄️ **Archived** | Superseded by another RFC |

---

## Current RFCs

### Active

| RFC | Title | Status | Author |
|-----|-------|--------|--------|
| [0001](text/0001-reactive-signals.md) | Reactive Signals | 💬 Discussion | @author |

*(This table will be updated as RFCs are submitted)*

### Implemented

*(None yet)*

### Rejected

*(None yet)*

---

## Tips for a Successful RFC

### ✅ Do

- **Start with a clear problem statement**
- **Provide concrete examples** (before/after code)
- **Consider edge cases** and error scenarios
- **Research prior art** (other languages, libraries)
- **Engage with feedback** constructively
- **Keep it focused** (one feature per RFC)

### ❌ Don't

- Propose multiple unrelated changes
- Ignore community feedback
- Skip rationale section
- Rush to implementation
- Take criticism personally

---

## Community Guidelines

RFCs are a collaborative process. When participating:

- 🤝 **Be respectful** of others' ideas
- 💡 **Provide constructive feedback**
- 📚 **Do your research** before commenting
- 🎯 **Stay on topic** in discussions
- ⏱️ **Be patient** - consensus takes time

See [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) for full guidelines.

---

## Questions?

- **RFC Process:** governance@velalang.org
- **Technical Questions:** [GitHub Discussions](https://github.com/velalang/vela/discussions)
- **Chat:** Discord (coming soon)

---

**Maintained by:** Vela Core Team  
**Last updated:** 2025-11-30
