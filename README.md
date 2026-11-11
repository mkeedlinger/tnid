# TNID

This repository contains the TNID specification.

TNIDs are UUIDv8-compatible identifiers with a human-readable name field and
compile-time type safety (where possible).

```
user.Br2flcNDfF6LYICnT
```

## Overview

Based on
[UUIDv8](https://datatracker.ietf.org/doc/html/rfc9562#name-uuid-version-8),
TNIDs maintain full UUID compatibility while adding:

- **20-bit name field** for runtime and compile-time type differentiation
- **Unambiguous, lexicographically sortable string representation** (unlike
  UUID's case-insensitive hex)
- **Two variants**: Variant 0 (time-sortable, like UUIDv7) and Variant 1
  (high-entropy, like UUIDv4)

TNIDs work anywhere a UUID is expected; all valid TNIDs are valid UUIDs.

## Project background

I made the main bits of TNID—the bit layout, the two current variants, and the
original core Rust implementation—by hand, before AI was very capable or common.
I’m not against AI. It’s helped review this project and even made some components
outright, like the site. But I didn’t “vibe” the standard or the core Rust
implementation.

## Resources

- [spec.md](./spec.md) - Full specification
- [tnid.info](https://tnid.info) - Website with illustrations and examples
- [tnid.info/implementations](https://tnid.info/implementations/) - Known
  implementations in various languages
- [Reference Implementation](https://github.com/mkeedlinger/tnid-rust) - A
  reference implementation in Rust
