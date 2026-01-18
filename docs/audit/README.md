# Audit Reports Archived

The comprehensive audit reports previously located in this directory have been moved to maintain documentation accuracy and relevance.

## New Location

**These audit reports are now located at:** [`docs/internals/archived-audits/`](../internals/archived-audits/)

## Reason for Move

The January 2026 audit reports contained:
- Specific line number references (e.g., `src/transport.rs:96-101`) that become outdated as code evolves
- Point-in-time verification dates and status assessments
- Temporal "modern practices" references tied to 2025-2026 timeframe
- Implementation completeness ratings that may no longer reflect current state

These characteristics make them historical snapshots rather than living documentation. They have been preserved in the archived internals directory where they serve as:
- Historical reference for understanding past implementation decisions
- Technical context for maintenance and refactoring
- Learning resources for understanding how systems were built
- Documentation of implementation patterns at a specific point in time

## Current Documentation

For up-to-date documentation, please refer to:
- Individual crate READMEs: `crates/praxis_*/README.md`
- Main documentation: `docs/README.md`
- Guides: `docs/guides/`
- Reference: `docs/reference/`
- Architecture: `docs/architecture.md`
- Example code: `examples/`
- API documentation: `cargo doc --workspace --no-deps --open`
