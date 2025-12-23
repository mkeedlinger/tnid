# TNID Design

TNIDs are based on
[UUIDv8](https://datatracker.ietf.org/doc/html/rfc9562#name-uuid-version-8).
**This means that TNIDs should be able to be used anywhere that expects a
standard UUID.**

The 67th and 68th bits (directly after the UUID variant field) are used to
denote the TNID variant, meaning there are 4 possible variants.

All TNIDs also have the first 5 nibbles (20 bits) reserved for a name.

## TNID variants

There are 2 bits that denote the TNID variant, allowing for 4 possible variants.

### Variant 0

Variant 0 is meant to be time sortable when sorted by its three representations
(u128, UUID hex, and TNID string). It's use case is similar to UUIDv7.

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
4. UUID variant (bits `0b10` per the UUIDv8 spec)\
   (2 bits)
5. TNID variant (bits `0b00` for variant 0)\
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
- Have the inner represenation (such as the time components) be useful after ID
  creation

#### Caveats

##### Collisions

Since the time components have millisecond precision, there is a chance of
collisions for TNIDs made in the same millisecond. With 57 random bits, if you
generate 10 billion TNIDv0 IDs per second, there's a .00000069% chance you'll
get a collision.

<details>
<summary>Math</summary>
% chance of collision = 10,000,000,000 / 2^57 * 100
</details>

##### Hex Sortability

The (very common) UUID hex format is not case sensitive, meaning that `0xa1` and
`0xA1` represent the same byte, _but those will not sort the same when compared
as hex strings_. Therefore, if you rely on the ability to time sort TNIDs when
represented in hex, ensure consistent casing. **(This is also the case with
UUIDs)**

Or better yet, always represent your TNIDs in an unambiguous format like a TNID
string or as a 128 uint (as Postgres does with its UUID type).

##### Future IDs

Since the time component only uses 43 bits to represent milliseconds, TNIDv0 IDs
can only be created until the year 2248. At that point what happens is
undefined, however implementations are encouraged to handle this case gracefully
(the reference Rust implementation chooses to allow the int to wrap around).

### Variant 1

Variant 1 is meant to have maximum randomness and entropy. Its use is similar to
UUIDv4.

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
4. UUID variant (bits 0b10 per the spec)\
   (2 bits)
5. TNID variant (bits 0b01 for variant 1)\
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

## TNID Name Encoding

TNIDs use a 5 bit character encoding. The ordering was specifically chosen such
that most systems would sort the character in a way that matches their byte
representation.

NOTE: after the null terminator, the rest of the name bits MUST be nulls.

| Bits  | Decimal | Char              |
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

## TNID Data Encoding

Since there are 102 bits of data that are _not_ the name and _not_ needed to
reproduce the TNID, they can be divided into 17 six-bit chunks. This lets us use
a base64<i>-like</i> encoding (but _not_
[RFC 4648 base64](https://datatracker.ietf.org/doc/html/rfc4648#section-4)).

Since the TNID data is exactly divisible into 6 bit chunks, there's no need to
handle padding.

| Bits   | Decimal | Char |
| ------ | ------- | ---- |
| 000000 | 0       | -    |
| 000001 | 1       | 0    |
| 000010 | 2       | 1    |
| 000011 | 3       | 2    |
| 000100 | 4       | 3    |
| 000101 | 5       | 4    |
| 000110 | 6       | 5    |
| 000111 | 7       | 6    |
| 001000 | 8       | 7    |
| 001001 | 9       | 8    |
| 001010 | 10      | 9    |
| 001011 | 11      | A    |
| 001100 | 12      | B    |
| 001101 | 13      | C    |
| 001110 | 14      | D    |
| 001111 | 15      | E    |
| 010000 | 16      | F    |
| 010001 | 17      | G    |
| 010010 | 18      | H    |
| 010011 | 19      | I    |
| 010100 | 20      | J    |
| 010101 | 21      | K    |
| 010110 | 22      | L    |
| 010111 | 23      | M    |
| 011000 | 24      | N    |
| 011001 | 25      | O    |
| 011010 | 26      | P    |
| 011011 | 27      | Q    |
| 011100 | 28      | R    |
| 011101 | 29      | S    |
| 011110 | 30      | T    |
| 011111 | 31      | U    |
| 100000 | 32      | V    |
| 100001 | 33      | W    |
| 100010 | 34      | X    |
| 100011 | 35      | Y    |
| 100100 | 36      | Z    |
| 100101 | 37      | _    |
| 100110 | 38      | a    |
| 100111 | 39      | b    |
| 101000 | 40      | c    |
| 101001 | 41      | d    |
| 101010 | 42      | e    |
| 101011 | 43      | f    |
| 101100 | 44      | g    |
| 101101 | 45      | h    |
| 101110 | 46      | i    |
| 101111 | 47      | j    |
| 110000 | 48      | k    |
| 110001 | 49      | l    |
| 110010 | 50      | m    |
| 110011 | 51      | n    |
| 110100 | 52      | o    |
| 110101 | 53      | p    |
| 110110 | 54      | q    |
| 110111 | 55      | r    |
| 111000 | 56      | s    |
| 111001 | 57      | t    |
| 111010 | 58      | u    |
| 111011 | 59      | v    |
| 111100 | 60      | w    |
| 111101 | 61      | x    |
| 111110 | 62      | y    |
| 111111 | 63      | z    |

## Overall Goals / Non-goals

This spec is meant to be useful in 99% of use cases. Admittedly, this means that
it will not work well for _all_ use cases. Particularly use cases generating
such massive volumes of IDs that the collision risk justifies sacrificing names
or other features for increased time precision or entropy.

Such cases are exceedingly rare, and TNIDs are designed to increase usablility
for the more common cases.
