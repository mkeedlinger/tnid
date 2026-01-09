# TNID

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

## Resources

- [Specification](./spec.md) - Complete TNID specification
- [Rust Implementation](https://github.com/mkeedlinger/tnid-rust) - Reference
  implementation and crate
- [Website](https://github.com/mkeedlinger/tnid-site) - Published at
  [tnid.info](https://tnid.info) with helpful illustrations
