# TNID Spec

Version: 0\
Status: Draft

TNIDs are based on
[UUIDv8](https://datatracker.ietf.org/doc/html/rfc9562#name-uuid-version-8).
**This means that TNIDs should be able to be used anywhere that expects a
standard UUID.** TNIDs follow the UUIDv8 specification for bit layout, byte
order (big-endian), and version/variant bits, ensuring full compatibility with
existing UUID infrastructure.

TNIDs include some extra features that developers may find useful:

- They include a name field, allowing ids of different kinds to be
  differentiated at runtime and (in languages that support it) at compile time.
- They have a string representation that is unambiguous, case-sensitive, and
  lexicographically sortable (unlike UUID's case insensitive hex
  representation).

## Terminology

The key words "MUST", "MUST NOT", "SHOULD", "SHOULD NOT", and "MAY" in this
document are to be interpreted as described in
[RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

**TNID Variant:** A scheme for interpreting the Payload bits. TNID uses the term
"variant" analogously to how UUID uses "version" to denote different structural
interpretations.

### Bit Terminology

TNIDs are 128 bits total, organized into the following components:

- **128 bits total**
  - **TNID-specific bits (122)**: All bits specific to TNID (not dictated by
    UUID)
    - **Name bits (20)**: The TNID name field
    - **Data bits (102)**: The bits encoded in the 17-character data portion of
      a TNID string
      - **TNID Variant bits (2)**: Selects which TNID variant is used
      - **Payload bits (100)**: Variant-specific data
  - **UUID-specific bits (6)**: Bits required by the UUIDv8 specification
    - **UUID Version (4)**
    - **UUID Variant (2)**

## Bit Layout

Below is the common aspects of all TNID variants.

`1111.1111.1111.1111.1111.5555.5555.5555`-

`5555.5555.5555.5555`-

`2222.5555.5555.5555`-

`3344.5555.5555.5555`-

`5555.5555.5555.5555.5555.5555.5555.5555.5555.5555.5555.5555`

1. Name\
   (20 bits) (5 nibbles = 4 encoded characters)\
   MUST be encoded using the [TNID Name Encoding](#tnid-name-encoding)
2. UUID version\
   (4 bits)\
   MUST be `0x8` for UUIDv8
3. UUID variant\
   (2 bits)\
   MUST be `0b10` per the UUIDv8 spec
4. TNID variant\
   (2 bits)\
   Denotes the TNID variant; decides how the data bits are used
5. TNID Data Bits These bits are available for use by each
   [TNID variant](#tnid-variants)\
   (100 bits)

## TNID variants

There are 2 bits that denote the TNID variant, allowing for 4 possible variants.

| TNID Variant | Similar to | Primary Use Case        | Key Feature          |
| ------------ | ---------- | ----------------------- | -------------------- |
| Variant 0    | UUIDv7     | Time-ordered IDs        | Millisecond sortable |
| Variant 1    | UUIDv4     | Maximum randomness      | 100 bits of entropy  |
| Variant 2    | -          | Reserved for future use | -                    |
| Variant 3    | -          | Reserved for future use | -                    |

Parsers SHOULD accept TNIDs with reserved variants.

### Variant 0

Variant 0 is meant to be time sortable when sorted by its three representations
(u128, UUID hex, and TNID string). Its use case and design is similar to UUIDv7.

Thus, it uses the TNID Data Bits to store (a) some time data and (b) some random
bits.

#### Layout

`1111.1111.1111.1111.1111.2222.2222.2222`-

`2222.2222.2222.2222`-

`3333.2222.2222.2222`-

`4455.2226.6666.6666`-

`6666.6666.6666.6666.6666.6666.6666.6666.6666.6666.6666.6666`

1. Name\
   (5 nibbles = 4 characters)
2. Milliseconds since 1970 unix epoch\
   (43 bits)
3. UUID version\
   (4 bits)
4. UUID variant (MUST be `0b10` per the UUIDv8 spec)\
   (2 bits)
5. TNID variant (MUST be `0b00` for variant 0)\
   (2 bits)
6. Random bits\
   (57 bits)

#### Goals & Non-goals

Goals:

- Be (millisecond precision) time sortable
- Be reasonable for 99% of use cases (low chance of collision, low stakes in
  case of collision)

Non-goals:

- Be useful where collision chances must be astronomically low
- Be useful for use cases generating extraordinary amounts of IDs in small time
  frames
- Have the inner representation (such as the time components) be useful after ID
  creation

#### Caveats

##### Collisions

Since the time components have millisecond precision, there is a chance of
collisions for TNIDs made in the same millisecond. With 57 random bits, if you
generate 1 million TNIDv0 IDs in the same millisecond, there's a ~0.00035%
chance of collision.

<details>
<summary>Math</summary>
Collision probability follows the birthday paradox: 1 - e^(-n²/2N) where N = 2^57

For 1 million IDs: 1 - e^(-(10^6)²/(2×2^57)) ≈ 0.00035%

</details>

##### Hex Sortability

The standard UUID hex representation is not case sensitive, meaning that `0xa1`
and `0xA1` represent the same byte, _but those will not sort the same when
compared as hex strings_. Therefore, if you rely on the ability to time sort
TNIDs when represented in hex, ensure consistent casing. **This is also the case
with UUIDs, or any use of hex encoded data.**

```
As strings:  "A" (ASCII 65) < "a" (ASCII 97)  →  "0xA1" < "0xa1"
As values:   0xA1 = 0xa1 = 161
```

Or better yet, always represent your TNIDs in an unambiguous format like a
[TNID string](#tnid-string) or as a 128 uint (as Postgres does with its UUID
type).

##### Future IDs

Since the time component only uses 43 bits to represent milliseconds, TNIDv0 IDs
can only represent times until approximately year 2248. After that, the
timestamp overflows. Implementations SHOULD allow the value to wrap rather than
error.

### Variant 1

Variant 1 is meant to have maximum randomness and entropy. Its use is similar to
UUIDv4.

#### Layout

`1111.1111.1111.1111.1111.2222.2222.2222`-

`2222.2222.2222.2222`-

`3333.2222.2222.2222`-

`4455.2222.2222.2222`-

`2222.2222.2222.2222.2222.2222.2222.2222.2222.2222.2222.2222`

1. Name\
   (5 nibbles = 4 characters)
2. Random Bits\
   (100 bits)
3. UUID version\
   (4 bits)
4. UUID variant (MUST be `0b10` per the spec)\
   (2 bits)
5. TNID variant (MUST be `0b01` for variant 1)\
   (2 bits)

#### Goals and Non-goals

Goals:

- Maximize entropy while conforming to UUID and TNID specs
- Be reasonable for 99% of use cases (low chance of collision, low stakes in
  case of collision)

Non-goals:

- Completely maximize entropy over TNID's other goals

#### Caveats

##### Collisions

Compared to UUIDv4, TNIDv1 has 22 fewer random bits (UUIDv4 has 122 entropy
bits, while TNIDv1 has 100). Assuming you created 100 billion IDs every second
for 20 years straight, you will use 0.00000029% of the possible addresses. For
most use cases this is expected to be vastly sufficient.

### Variant 2

Reserved for future definition.

### Variant 3

Reserved for future definition.

## Representations

TNIDs are 128 bits and can be represented any way a UUID can (hex string, bytes,
integer, etc.). TNIDs also define their own string format with advantages over
UUID's
[typical hex representation](https://datatracker.ietf.org/doc/html/rfc9562#name-uuid-format).

<details>
<summary>Example: The same TNID in different formats</summary>

A TNID with name "test" and variant 1:

| Format             | Value                                             |
| ------------------ | ------------------------------------------------- |
| TNID string        | `test.x8MRU0xetVa6QZeZR`                          |
| u128 hex           | `0xCAB19F495DC78C1F9AB98261DB92A91C`              |
| UUID hex           | `cab19f49-5dc7-8c1f-9ab9-8261db92a91c`            |
| Bytes (big-endian) | `CA B1 9F 49 5D C7 8C 1F 9A B9 82 61 DB 92 A9 1C` |

</details>

### TNID String

`<name>.<encoded-data>`

**name**: The TNID name as ascii chars. MUST be 1 to 4 of the
[allowed TNID Name Encoding characters](#tnid-name-encoding).

**encoded-data**: The [TNID Data Encoding](#tnid-data-encoding) of the (1) TNID
variant and (2) the TNID Data Bits (see [layout](#bit-layout)). These are taken
in the order they appear: the first 40 data bits, (skipping the UUID version
bits) then the 2 TNID variant bits, (skipping the UUID variant bits) then the
remaining 60 data bits. MUST be 17 characters.

Example: `test.Br2flcNDfF6LYICnT`

## Encodings

These are the encodings that are used for TNID representations. Both are
designed such that the ordering of the bit representation matches the ascii
character representation.

An attempt was made to use only "safe" characters. For example, all characters
used in both encodings are URL safe (unreserved characters per
[RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-2.2)), meaning TNIDs
can be used in URLs without percent-encoding.

### TNID Name Encoding

TNIDs use a 5 bit character encoding. The character ordering (0-4, then a-z) was
specifically chosen to match ASCII lexicographic sorting, ensuring that TNID
names sort correctly as both encoded bits and as strings. For example, name "a"
< "b" < "z" both as characters and in their encoded bit representation.

The name MUST contain at least one non-null character.

If a name is less than the maximum 4 characters, then there MUST be nulls
filling in the unused space at the end (least significant bits). For example, if
a name `ab` was encoded, then the first 10 bits (most significant) would be the
encoded chars, and the remaining 10 bits would all be zeroes (nulls).

```
"ab" encoded (20 bits):

✓ Valid:   [00110][00111][00000][00000]
              a      b    null   null

✗ Invalid: [00110][00000][00111][00000]
              a    null     b    null
           (non-null after null is invalid)
```

#### Mapping

| Bits  | Decimal | Char (ascii)      |
| ----- | ------- | ----------------- |
| 00000 | 0       | (null-terminator) |
| 00001 | 1       | 0                 |
| 00010 | 2       | 1                 |
| 00011 | 3       | 2                 |
| 00100 | 4       | 3                 |
| 00101 | 5       | 4                 |
| 00110 | 6       | a                 |
| 00111 | 7       | b                 |
| 01000 | 8       | c                 |
| 01001 | 9       | d                 |
| 01010 | 10      | e                 |
| 01011 | 11      | f                 |
| 01100 | 12      | g                 |
| 01101 | 13      | h                 |
| 01110 | 14      | i                 |
| 01111 | 15      | j                 |
| 10000 | 16      | k                 |
| 10001 | 17      | l                 |
| 10010 | 18      | m                 |
| 10011 | 19      | n                 |
| 10100 | 20      | o                 |
| 10101 | 21      | p                 |
| 10110 | 22      | q                 |
| 10111 | 23      | r                 |
| 11000 | 24      | s                 |
| 11001 | 25      | t                 |
| 11010 | 26      | u                 |
| 11011 | 27      | v                 |
| 11100 | 28      | w                 |
| 11101 | 29      | x                 |
| 11110 | 30      | y                 |
| 11111 | 31      | z                 |

### TNID Data Encoding

This encoding is used for the data portion of a [TNID String](#tnid-string)
(after the `.`). To reconstruct the full 128-bit TNID from a string, you need
the TNID Variant and TNID Data bits (see [layout](#bit-layout)). The name
appears before the `.`, and the UUID version/variant are constants dictated by
this spec (and the UUID spec that this complies with). This leaves 102 bits,
which divides evenly into 17 six-bit chunks (102 = 17 × 6), requiring no
padding.

These 17 chunks are encoded using a base64-_like_ encoding (but _not_
[RFC 4648 base64](https://datatracker.ietf.org/doc/html/rfc4648#section-4)).
Below, each symbol (1-9, A-H) represents one of the 17 encoded characters. Since
6-bit characters don't align with 4-bit nibbles, they overlap at boundaries:

`nnnn.nnnn.nnnn.nnnn.nnnn.1111.1122.2222`-

`3333.3344.4444.5555`-

`vvvv.5566.6666.7777`-

`uutt.8888.8899.9999`-

`AAAA.AABB.BBBB.CCCC.CCDD.DDDD.EEEE.EEFF.FFFF.GGGG.GGHH.HHHH`

- n = Name (20 bits)
- v = UUID version (4 bits, skipped)
- u = UUID variant (2 bits, skipped)
- t = TNID variant (2 bits, last 2 bits of character 7)
- 1-9, A-H = The 17 encoded characters (6 bits each)

#### Mapping

| Bits   | Decimal | Char (ascii) |
| ------ | ------- | ------------ |
| 000000 | 0       | -            |
| 000001 | 1       | 0            |
| 000010 | 2       | 1            |
| 000011 | 3       | 2            |
| 000100 | 4       | 3            |
| 000101 | 5       | 4            |
| 000110 | 6       | 5            |
| 000111 | 7       | 6            |
| 001000 | 8       | 7            |
| 001001 | 9       | 8            |
| 001010 | 10      | 9            |
| 001011 | 11      | A            |
| 001100 | 12      | B            |
| 001101 | 13      | C            |
| 001110 | 14      | D            |
| 001111 | 15      | E            |
| 010000 | 16      | F            |
| 010001 | 17      | G            |
| 010010 | 18      | H            |
| 010011 | 19      | I            |
| 010100 | 20      | J            |
| 010101 | 21      | K            |
| 010110 | 22      | L            |
| 010111 | 23      | M            |
| 011000 | 24      | N            |
| 011001 | 25      | O            |
| 011010 | 26      | P            |
| 011011 | 27      | Q            |
| 011100 | 28      | R            |
| 011101 | 29      | S            |
| 011110 | 30      | T            |
| 011111 | 31      | U            |
| 100000 | 32      | V            |
| 100001 | 33      | W            |
| 100010 | 34      | X            |
| 100011 | 35      | Y            |
| 100100 | 36      | Z            |
| 100101 | 37      | _            |
| 100110 | 38      | a            |
| 100111 | 39      | b            |
| 101000 | 40      | c            |
| 101001 | 41      | d            |
| 101010 | 42      | e            |
| 101011 | 43      | f            |
| 101100 | 44      | g            |
| 101101 | 45      | h            |
| 101110 | 46      | i            |
| 101111 | 47      | j            |
| 110000 | 48      | k            |
| 110001 | 49      | l            |
| 110010 | 50      | m            |
| 110011 | 51      | n            |
| 110100 | 52      | o            |
| 110101 | 53      | p            |
| 110110 | 54      | q            |
| 110111 | 55      | r            |
| 111000 | 56      | s            |
| 111001 | 57      | t            |
| 111010 | 58      | u            |
| 111011 | 59      | v            |
| 111100 | 60      | w            |
| 111101 | 61      | x            |
| 111110 | 62      | y            |
| 111111 | 63      | z            |

## Overall Goals / Non-goals

This spec is meant to be useful in 99% of use cases. Admittedly, this means that
it will not work well for _all_ use cases. Particularly use cases generating
such massive volumes of IDs that the collision risk justifies sacrificing names
or other features for increased time precision or entropy.

Such cases are exceedingly rare, and TNIDs are designed to increase usablility
for the more common cases.
