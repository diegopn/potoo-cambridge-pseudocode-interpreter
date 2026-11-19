<p align="center">
  <img src="https://diegopn.github.io/potoo-site/src/assets/images/language-interpreter-icon.svg?v=1c651ca7" alt="Potoo IDE language interpreter icon" width="120" />
</p>

<h1 align="center">Cambridge Pseudocode Interpreter</h1>

<p align="center">A Rust interpreter for Cambridge International AS & A Level Computer Science 9618 pseudocode.</p>

<p align="center">
  <a href="https://github.com/diegopn/potoo-cambridge-pseudocode-interpreter/releases">Download</a> ·
  <a href="https://diegopn.github.io/potoo-site/manual/cambridge-pseudocode/">Website</a> ·
  <a href="https://diegopn.github.io/potoo-site/manual/cambridge-pseudocode/">User manual</a>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white"></a>
  <a href="https://doc.rust-lang.org/cargo/"><img alt="Cargo" src="https://img.shields.io/badge/Cargo-DEA584?style=for-the-badge&logo=rust&logoColor=white"></a>
  <img alt="Rust library" src="https://img.shields.io/badge/Rust_library-4A86CF?style=for-the-badge">
  <img alt="CLI" src="https://img.shields.io/badge/CLI-6A4C93?style=for-the-badge">
</p>

This Rust package is part of the Potoo IDE ecosystem and is distributed independently from the desktop interface. Integration with the IDE has not been implemented or verified. Cambridge pseudocode is an assessment notation, not a general-purpose programming language.

The crate supports two explicit guide profiles: examinations in 2026 and examinations in 2027–2029.

## Implemented subset

- Explicit edition selection through `Edition::Cambridge2026` or `Edition::Cambridge2027To2029`.
- Case-insensitive identifiers, scalar declarations, global and routine-local literal constants, and fixed one- or multi-dimensional arrays with declared lower bounds.
- Integer, real, character, string, Boolean, and calendar-validated date values; arithmetic, `DIV`, `MOD`, comparisons, `AND`, `OR`, `NOT`, and string concatenation with `&`.
- `INPUT`, `OUTPUT`, `IF` / `ELSE`, `CASE`, `FOR` / `NEXT`, `WHILE` / `ENDWHILE`, and `REPEAT` / `UNTIL`.
- Text and random file statements: `OPENFILE`, `READFILE`, `WRITEFILE`, `EOF`, `CLOSEFILE`, `SEEK`, `GETRECORD`, and `PUTRECORD`, routed through the `Host` interface.
- User-defined record types with scalar and array fields, arrays of records, nested dot-field access, same-type assignment, enumerated types with declaration-order comparisons and integer offsets, set types with named constant definitions, and typed pointers with address-of and dereference syntax.
- Classes with typed properties, public-by-default or explicit `PUBLIC` / `PRIVATE` access, procedures and functions as methods, `NEW` constructors, single inheritance, parent-method lookup, and `SUPER` calls.
- Procedures with `BYVAL`/`BYREF` parameters and functions with `BYVAL` parameters, local declarations, returns, and recursion. `BYREF` parameters alias caller variables, record fields, or array elements during execution; repeated references to the same location share updates immediately.
- `LENGTH`, `RIGHT`, `MID`, `LCASE`, `UCASE`, `INT`, and host-provided `RAND`.

The parser records the chosen guide edition in the program and execution result. The two currently documented guides share this implemented syntax; edition selection is still required so programs keep the intended examination context explicit.

## Current limits

Set declarations and `DEFINE` constants are supported; set membership and mutation syntax is not defined in the selected guides. Edition-specific syntax beyond the shared core is not implemented. The CLI implements text-file modes; random-record operations require an embedding host to implement the `Host` file methods. The host controls paths, permissions, text encoding, and random-record storage. Date literals and scalar input use the documented `dd/mm/yyyy` format and are calendar-validated. The guides do not define text serialization formats for arrays, records, sets, pointers, or objects. The default `Host::input_value` parses scalar values from `input()` and requires a host override for compound values; the interpreter validates returned values against the declared type. Pointer input must provide an address meaningful to the current interpreter. Object assignment requires the same declared class. Addresses of routine-local variables expire when the routine returns. See [reference coverage](REFERENCE-COVERAGE.md) for these boundaries.

## Library use

Choose an edition, call `parse(source, edition)`, then `check(&program)` and `Interpreter::new(host).run(&program)`. Implement `Host` to supply input, output, and random values. Override `Host::input_value` when the host needs to provide an array, record, set, object, or pointer value.

## Command line

```sh
cargo run --locked -- --edition 2026 source.pseudo
cargo run --locked -- --edition 2027-2029 source.pseudo
```

The CLI reads a UTF-8 source file and connects `INPUT`, `OUTPUT`, and `RAND` to its host.

## References

- [Cambridge 9618 pseudocode guide for 2026](https://www.cambridgeinternational.org/Images/697401-2026-pseudocode-guide-for-teachers.pdf)
- [Cambridge 9618 pseudocode guide for 2027–2029](https://www.cambridgeinternational.org/Images/721401-2027-2029-pseudocode-guide.pdf)
- [Potoo Cambridge pseudocode manual](https://diegopn.github.io/potoo-site/manual/cambridge-pseudocode/)

## Support

<p align="left">
  <a href="https://github.com/sponsors/diegopn"><img alt="Become a sponsor" src="https://img.shields.io/badge/Become_a_sponsor-EA4AAA?style=for-the-badge&logo=githubsponsors&logoColor=white"></a>
</p>

## Project policies

- [Code of conduct](CODE_OF_CONDUCT.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [GPL-3.0-or-later license](LICENSE)
