# Governance

This document outlines the governance model for the Vela programming language project.

---

## Table of Contents

- [Overview](#overview)
- [Project Vision](#project-vision)
- [Core Team](#core-team)
- [Decision-Making Process](#decision-making-process)
- [RFC (Request for Comments) Process](#rfc-request-for-comments-process)
- [Release Process](#release-process)
- [Trademark Policy](#trademark-policy)
- [Community Roles](#community-roles)
- [Transparency and Communication](#transparency-and-communication)
- [Conflict Resolution](#conflict-resolution)
- [Amendments](#amendments)

---

## Overview

Vela is an open-source project committed to transparent, collaborative development. This governance model aims to:

- 🎯 **Maintain project direction** while welcoming community input
- 🤝 **Foster inclusive decision-making** with clear processes
- 🚀 **Enable rapid innovation** without sacrificing stability
- 📢 **Ensure transparency** in all major decisions

---

## Project Vision

**Mission:** Build a modern, reactive programming language that makes cross-platform development simple, safe, and performant.

**Core Values:**
1. **Developer Experience First:** Tools and language should feel intuitive
2. **Safety Without Compromise:** Memory-safe, type-safe, thread-safe by default
3. **Performance Matters:** Zero-cost abstractions, LLVM optimization
4. **Community-Driven:** Open development, welcoming contributions
5. **Multi-Platform from Day One:** Native, web, mobile, desktop support

---

## Core Team

### Structure

The Core Team consists of individuals who have demonstrated sustained contributions and commitment to the project.

#### Core Team Roles

**1. Project Lead**
- **Responsibilities:**
  - Final decision-making authority on contentious issues
  - Strategic direction and roadmap planning
  - External representation of the project
  - Appointing and removing Core Team members
- **Term:** Indefinite, with annual community review

**2. Technical Leads**
- **Areas:**
  - Compiler & Language Design
  - Type System & Semantics
  - Reactive System & Runtime
  - Tooling (CLI, LSP, DevTools)
  - Standard Library
- **Responsibilities:**
  - Technical decisions in their area
  - RFC review and approval
  - Code review and quality assurance
  - Mentoring contributors
- **Term:** 2 years, renewable

**3. Release Manager**
- **Responsibilities:**
  - Release planning and coordination
  - Version numbering and changelog
  - Security updates and patches
  - Compatibility testing
- **Term:** 1 year, renewable

**4. Community Manager**
- **Responsibilities:**
  - Community engagement and support
  - Documentation coordination
  - Event organization
  - Social media and outreach
- **Term:** 1 year, renewable

### Current Core Team

| Name | Role | GitHub | Email |
|------|------|--------|-------|
| TBD | Project Lead | @username | lead@velalang.org |
| TBD | Compiler Lead | @username | compiler@velalang.org |
| TBD | Type System Lead | @username | types@velalang.org |
| TBD | Tooling Lead | @username | tooling@velalang.org |
| TBD | Release Manager | @username | release@velalang.org |
| TBD | Community Manager | @username | community@velalang.org |

### Becoming a Core Team Member

**Criteria:**
- 6+ months of sustained, high-quality contributions
- Deep knowledge of relevant codebase area
- Demonstrated commitment to project values
- Strong communication and collaboration skills
- Community endorsement

**Process:**
1. Nomination by existing Core Team member
2. 2-week community comment period
3. Core Team vote (75% approval required)
4. Public announcement

---

## Decision-Making Process

### Decision Categories

#### 1. **Routine Decisions** (No formal process)
- Bug fixes
- Documentation improvements
- Minor refactoring
- Test additions

**Process:** PR review and merge by any Core Team member

#### 2. **Significant Decisions** (RFC required)
- New language features
- Breaking changes
- Major architectural changes
- Standard library additions

**Process:** RFC (see RFC Process section)

#### 3. **Strategic Decisions** (Core Team vote)
- Project roadmap changes
- Governance changes
- Trademark policy
- Major partnerships

**Process:**
1. Proposal by Core Team member
2. 1-week discussion period
3. Core Team vote (2/3 majority required)
4. Public announcement

### Voting

- **Quorum:** 50% of Core Team members must participate
- **Approval Thresholds:**
  - Routine: 1 Core Team member approval
  - RFC: 2 Technical Lead approvals
  - Strategic: 2/3 Core Team majority
  - Governance: 75% Core Team super-majority

---

## RFC (Request for Comments) Process

See [RFC Process](#) for detailed information. Summary:

### RFC Lifecycle

```
Draft → Discussion → Core Review → Accepted/Rejected → Implementation
```

### When to Write an RFC

- New language syntax or semantics
- Breaking changes to APIs
- Major compiler optimizations
- New standard library modules
- Tooling architecture changes

### RFC Format

See `vela-rfcs/0000-template.md` for template.

**Required Sections:**
- Summary
- Motivation
- Detailed Design
- Rationale and Alternatives
- Unresolved Questions
- Future Possibilities

### Timeline

- **Draft:** Author writes initial proposal
- **Discussion:** 2-4 weeks community feedback
- **Core Review:** 1-2 weeks Technical Lead review
- **Decision:** Accepted or Rejected with rationale
- **Implementation:** Feature flag → stabilization

---

## Release Process

### Versioning

Vela follows **Semantic Versioning 2.0.0**:

- **Major (X.0.0):** Breaking changes
- **Minor (0.X.0):** New features (backward compatible)
- **Patch (0.0.X):** Bug fixes

### Release Channels

1. **Nightly:** Daily builds, experimental features
2. **Beta:** Pre-release testing (feature-complete)
3. **Stable:** Production-ready releases
4. **LTS (Long-Term Support):** Extended support (future)

### Release Cadence

- **Major:** Every 12-18 months
- **Minor:** Every 6-8 weeks
- **Patch:** As needed (security, critical bugs)

### Release Checklist

1. ✅ All RFCs implemented
2. ✅ Test suite passing (100% critical paths)
3. ✅ Documentation updated
4. ✅ Changelog generated
5. ✅ Security audit (for major/minor)
6. ✅ Beta testing period (2 weeks for major, 1 week for minor)
7. ✅ Release notes published
8. ✅ Announcement on all channels

---

## Trademark Policy

### Ownership

The "Vela" name and logo are trademarks owned by [TBD: Foundation or Organization].

### Allowed Uses

**✅ Permitted without permission:**
- Referring to the Vela programming language
- Describing compatibility ("works with Vela")
- Unmodified redistribution of official binaries
- Community events ("Vela Meetup")

**⚠️ Requires permission:**
- Modified versions of Vela ("ForkVela")
- Commercial products using Vela name
- Domain names containing "vela"
- Merchandise with Vela branding

### Contact

For trademark questions: legal@velalang.org

---

## Community Roles

### Contributors

Anyone who submits a PR, files an issue, or participates in discussions.

**Recognition:**
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Eligibility for contributor rewards (future)

### Collaborators

Frequent contributors with commit access to specific repositories.

**Criteria:**
- 3+ months of regular contributions
- Demonstrated technical expertise
- Endorsed by Core Team member

**Privileges:**
- Push access to assigned repositories
- Issue triage and labeling
- PR review and merge (for routine changes)

### Emeritus Core Team

Former Core Team members who have stepped down but remain honored advisors.

**Privileges:**
- Advisory role in strategic decisions
- Invited to Core Team meetings
- Recognition on website

---

## Transparency and Communication

### Public Channels

- **GitHub Issues:** Bug reports, feature requests
- **GitHub Discussions:** General questions, ideas
- **Discord:** Real-time chat (coming soon)
- **Blog:** Announcements, technical posts
- **Twitter:** Updates and news

### Core Team Meetings

- **Frequency:** Bi-weekly
- **Format:** Video call, recorded (audio only)
- **Agenda:** Published 3 days in advance
- **Notes:** Published within 48 hours

### Roadmap

Public roadmap maintained at [docs.velalang.org/roadmap](https://docs.velalang.org/roadmap):

- Quarterly goals
- Current sprint items
- Planned features
- RFC status

---

## Conflict Resolution

### Process

1. **Direct Discussion:** Parties attempt to resolve privately
2. **Mediation:** Community Manager facilitates discussion
3. **Core Team Review:** Issue escalated to Core Team
4. **Project Lead Decision:** Final authority if no consensus

### Code of Conduct Violations

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for enforcement process.

---

## Amendments

### Proposing Changes

1. Open GitHub issue with `governance` label
2. Describe proposed change and rationale
3. 2-week community comment period
4. Core Team vote (75% approval required)
5. Update this document and announce

### History

- **v1.0 (2025-11-30):** Initial governance model

---

## Questions?

For governance questions:

- **Email:** governance@velalang.org
- **GitHub:** [github.com/velalang/vela/discussions](https://github.com/velalang/vela/discussions)

---

*Last updated: 2025-11-30*  
*Version: 1.0*
