# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/SecurityRonin/shrinkpath/compare/shrinkpath-v0.1.1...shrinkpath-v0.1.2) - 2026-07-25

### Added

- *(ts)* add fs-aware module — filesystem disambiguation and git root
- *(ts)* add public API — shrink, shrinkDetailed, convenience functions
- *(ts)* add hybrid strategy — graduated 4-phase shortening
- *(ts)* add unique strategy — prefix disambiguation
- *(ts)* add ellipsis strategy — budget-aware shortening
- *(ts)* add fish strategy — segment abbreviation
- *(ts)* add path-info module — parsing and segment classification
- *(ts)* add platform module — path style detection

### Documentation

- reverse-write PRD + ADRs; mkdocs excludes governance docs (fleet standard)
- add MkDocs site with privacy/terms (fleet standard)
- add implementation plan for TypeScript npm port
- add design spec for TypeScript npm port

### Fixed

- *(vet)* declare own crates first-party so version bumps don't break supply-chain audit
- *(clippy)* use sort_by_key(Reverse) for mapped-location ordering
- *(ts)* add @types/node, fix exports condition ordering

### Other

- *(ts)* drop dead post-phase4 collapse; cover hybrid/ellipsis/unique to 100%
# Changelog
