# TNID Design

TNIDs are based on
[UUIDv8](https://datatracker.ietf.org/doc/html/rfc9562#name-uuid-version-8),
except that the 67th and 68th bits (directly after the UUID variant field) are
used to denote the TNID variant.

This means that TNIDs should be able to be used anywhere that expects a standard
UUID.

All TNIDs also have the first 5 nibbles (20 bits) reserved for a name. The
binary encoding _can_ be different between TNID variants.

## TNID variants

There are 2 bits that denote the TNID variant, allowing for 4 possible variants.

### Variant 0

Variant 0 is meant to be time sortable when sorted by its three representations
(u128, UUID hex, and TNID string). It's use case is similar to UUIDv7.

`1111.1111.1111.1111.1111.2222.2222.3333`-

`3333.3333.3333.3333`-

`4444.3333.3777.7777`-

`5566.7777.7777.7777`-

`7777.7777.7777.7777.7777.7777.7777.7777.7777.7777.7777.7777`

1. Name (5 nibbles = 4 chars)
2. Years since 1970 (up to 2^8 = 256 years)
3. Seconds in year (25 bits)
4. UUID version
5. UUID variant (bits 0b10 per the spec)
6. Typed ID variant (bits 0b00 for variant 0)
7. Random bits

### Goals & Non goals

Goals:

- Be (second precision) time sortable
- Be reasonable for 99% of use cases (low chance of collision, low stakes in
  case of collision)

Non goals:

- Be useful where collision chances must be astronomically low
- Be useful for use cases generating extraordinary amounts of IDs in small time
  frames
- Have the inner represenation (such as the time components) be useful after ID
  creation

### Name encoding

_todo_

### Caveats

#### Collisions

Since the time components have second precision, there is a chance of collisions
for TNIDs made in the same second. With 67 random bits, if you generate 100
billion TNIDv0 IDs per second, there's roughly a one in a billion chance you'll
get a collision.

#### Hex Sortability

The common UUID hex format is not case sensitive, meaning that `0xa1` and `0xA1`
represent the same byte, _but those will not sort the same when compared as hex
strings_. Therefore, if you rely on the ability to time sort TNIDs when
represented in hex, ensure consistent casing. (This is also the case with UUIDs)

Or better yet, always represent your TNI IDs in an unambiguous format like a TNI
string or as a 128 uint (as Postgres does with its UUID type).

#### Future IDs

Since the `years since 1970` component only uses a single byte, TNIDv0 IDs can
only be created until the year 2225. At that point what happens is undefined,
however implementations are encouraged to handle this case gracefully (the
reference Rust implementation chooses to allow the int to overflow).

### Variant 1

Variant 1 is meant to have maximum randomness and entropy. Its use is similar to
UUIDv4.

`1111.1111.1111.1111.1111.2222.2222.2222`-

`2222.2222.2222.2222`-

`3333.2222.2222.2222`-

`4455.2222.2222.2222`-

`2222.2222.2222.2222.2222.2222.2222.2222.2222.2222.2222.2222`

1. Name (5 nibbles = 4 chars)
2. Random Bits
3. UUID version
4. UUID variant (bits 0b10 per the spec)
5. TNID variant (bits 0b01 for variant 1)

## Name Encodings

### Encoding 0

Encoding 0 is a 5 bit character encoding. The mapping is as below:
