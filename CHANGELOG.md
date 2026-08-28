# Changelog

All notable changes to this project are documented in this file.

## [0.4.0] - 2026-08-28

### Removed

- Indentation-based blocks. Every block now needs an explicit `do ... end`.
- Experimental type refinement (the `type`/`when` conditional type alias feature).
- `is`, `isa`, `isn`, `isnta`, and several binary-operator tokens.
- `vararg` support.
- The explicit `handle` keyword. Handling is now inferred from `!` followed by match cases.

### Added

- Explicit `trait` keyword, now distinct from `type`.
- Experimental machine-code backend (`--bin`/`--asm`), compiling a small subset of Mamba via Cranelift, bypassing Python.
- Classes now automatically inherit all fields and functions from their parent.

### Changed

- `for` loop bodies no longer leak or shadow outer variables. Reassigning an outer mutable
  variable is still allowed.
- Parse errors now show the actual offending token.

### Fixed

- A class or trait could inherit from a tuple or function type. This is now rejected.
- The checker now correctly tracks which variable was assigned to across `if`/`match` branches.
- Default-argument type inference used Python-side type names instead of Mamba-side ones.

### Internal

- Reintroduced coverage reporting in CI.
- Added githooks, a Nix dev environment, and other tooling improvements.
- Removed dead code, parameterized more of the test suite.
