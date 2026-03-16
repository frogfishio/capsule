# Capsule File Format Specification

Status: Draft
Version: 0001

1. Introduction

Capsule is a file container format consisting of:

- a fixed-width ASCII prelude;
- a header block; and
- a payload block.

The prelude identifies the file as a Capsule, declares the format version,
declares the encoding used for the header and payload, and declares the
encoded length of the header block.

The header block immediately follows the prelude.
The payload block immediately follows the header block and extends to end of
file.

2. Conventions and Definitions

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", "SHOULD NOT", and
"MAY" in this document are to be interpreted as described in RFC 2119 and
RFC 8174.

For the purposes of this specification:

- "byte" means an 8-bit octet.
- "ASCII" means US-ASCII.
- "uppercase hexadecimal" means the ASCII characters 0-9 and A-F only.
- "encoded bytes" means the exact bytes as stored in the file, before any
  decoding of the selected encoding.
- "end of file" means the final byte of the Capsule file.

3. File Structure

A Capsule file MUST consist of the following sequence of fields, in order:

1. Magic
2. Version
3. Encoding
4. Header Length
5. Header Block
6. Payload Block

The first four fields form the prelude.

4. Prelude

The prelude MUST appear at the beginning of the file and MUST be exactly
16 bytes long.

4.1. Magic

The Magic field MUST consist of exactly 7 bytes and MUST be the literal
ASCII string:

CAPSULE

A parser MUST reject the file if the Magic field does not exactly match this
value.

4.2. Version

The Version field MUST consist of exactly 4 bytes.

These 4 bytes MUST be uppercase ASCII hexadecimal characters representing an
unsigned 16-bit integer.

The Version field MUST represent a value in the inclusive range 0001 to FFFF.

The value 0000 is RESERVED and MUST NOT be used.

A parser MUST reject the file if:

- the Version field contains any byte outside 0-9 or A-F;
- the Version field is lowercase hexadecimal; or
- the Version field decodes to 0000.

4.3. Encoding

The Encoding field MUST consist of exactly 1 byte.

The Encoding field MUST be one of the following ASCII characters:

- A
- B
- C

These values are defined as follows:

- A = ASCII
- B = Base64
- C = CBOR

The Encoding field applies to both the Header Block and the Payload Block.

The Encoding field MUST NOT apply to the prelude.

A parser MUST reject the file if the Encoding field contains any value other
than A, B, or C.

4.4. Header Length

The Header Length field MUST consist of exactly 4 bytes.

These 4 bytes MUST be uppercase ASCII hexadecimal characters representing an
unsigned 16-bit integer.

The Header Length field specifies the number of encoded bytes in the Header
Block as stored in the file.

The Header Length value MAY be 0000.

A parser MUST reject the file if the Header Length field contains any byte
outside 0-9 or A-F or if lowercase hexadecimal is used.

5. Body

The body MUST immediately follow the 16-byte prelude.

5.1. Header Block

The Header Block MUST consist of exactly Header Length bytes.

The Header Block MUST be interpreted according to the Encoding field.

If Header Length is 0000, the Header Block is empty.

5.2. Payload Block

The Payload Block MUST consist of all remaining bytes in the file after the
Header Block.

The Payload Block MAY be empty.

The Payload Block MUST be interpreted according to the Encoding field.

6. Encoding Semantics

6.1. ASCII Encoding

If Encoding is A:

- the Header Block MUST be ASCII data;
- the Payload Block MUST be ASCII data.

A parser SHOULD reject non-ASCII bytes in either block when Encoding is A.

6.2. Base64 Encoding

If Encoding is B:

- the Header Block MUST be Base64-encoded data;
- the Payload Block MUST be Base64-encoded data.

This specification does not permit lowercase or alternative alphabets unless
a future version explicitly defines them.

An implementation SHOULD define whether embedded whitespace is accepted.
For strict interoperability, an implementation SHOULD reject embedded
whitespace unless otherwise specified by profile or agreement.

6.3. CBOR Encoding

If Encoding is C:

- the Header Block MUST be CBOR data;
- the Payload Block MUST be CBOR data.

This specification defines framing only.
Unless otherwise specified by a higher-level profile, the internal structure
and meaning of CBOR content are application-defined.

7. Parsing Procedure

A Capsule parser MUST perform the following steps in order:

1. Read exactly 16 bytes from the beginning of the file as the prelude.
2. Verify that bytes 0 through 6 are the ASCII string CAPSULE.
3. Parse bytes 7 through 10 as the Version field.
4. Parse byte 11 as the Encoding field.
5. Parse bytes 12 through 15 as the Header Length field.
6. Decode the Header Length field as an unsigned 16-bit integer value L.
7. Read exactly L bytes as the Header Block.
8. Treat all remaining bytes as the Payload Block.

A parser MUST reject the file if:

- the file contains fewer than 16 bytes;
- any prelude field violates this specification; or
- fewer than L bytes remain in the file for the Header Block.

8. Serialization Procedure

A Capsule writer MUST construct the file in the following order:

1. write the literal ASCII string CAPSULE;
2. write the Version field as 4 uppercase ASCII hexadecimal characters;
3. write the Encoding field as exactly one of A, B, or C;
4. write the Header Length field as 4 uppercase ASCII hexadecimal characters,
   where the value equals the number of encoded bytes in the Header Block;
5. write the Header Block;
6. write the Payload Block.

A writer MUST ensure that the Header Length field exactly matches the number
of encoded bytes written for the Header Block.

9. Field Layout Summary

The on-disk layout of a Capsule file is:

- 7 bytes: Magic
- 4 bytes: Version
- 1 byte: Encoding
- 4 bytes: Header Length
- L bytes: Header Block
- remaining bytes: Payload Block

Total prelude size: 16 bytes.

10. ABNF

The following ABNF describes the Capsule prelude and overall framing.
ABNF is expressed using RFC 5234 notation.

CAPSULE-FILE = PRELUDE HEADER-BLOCK PAYLOAD-BLOCK

PRELUDE      = MAGIC VERSION ENCODING HEADER-LENGTH

MAGIC        = %x43.41.50.53.55.4C.45
             ; "CAPSULE"

VERSION      = 4HEXDIG-U
             ; uppercase ASCII hex, 0001-FFFF

ENCODING     = %x41 / %x42 / %x43
             ; A / B / C

HEADER-LENGTH = 4HEXDIG-U
              ; uppercase ASCII hex, 0000-FFFF

HEADER-BLOCK = *OCTET
             ; exactly Header Length bytes

PAYLOAD-BLOCK = *OCTET
              ; all remaining bytes

HEXDIG-U     = DIGIT / %x41 / %x42 / %x43 / %x44 / %x45 / %x46
             ; 0-9 / A-F

OCTET        = %x00-FF

ABNF alone does not express the following semantic constraints, which are
normative:

- PRELUDE MUST be exactly 16 bytes.
- VERSION MUST decode to a value in the range 0001-FFFF.
- HEADER-LENGTH MUST decode to a value in the range 0000-FFFF.
- HEADER-BLOCK length MUST equal the decoded HEADER-LENGTH value.
- PAYLOAD-BLOCK consumes all bytes remaining after HEADER-BLOCK.

11. Interoperability Notes

Implementations SHOULD treat the prelude as case-sensitive.

Implementations SHOULD generate uppercase hexadecimal in all numeric fields.

Implementations MUST NOT infer header length from delimiters, line endings,
or decoding results. Header length is determined exclusively by the Header
Length field.

Implementations MUST measure Header Length using encoded bytes as stored in
the file, not decoded bytes.

12. Security Considerations

Implementations SHOULD validate that the declared Header Length does not
exceed the available bytes in the file.

Implementations SHOULD bound resource usage when decoding Base64 or CBOR
content.

Implementations SHOULD NOT assume that header or payload content is safe,
trusted, or schema-valid solely because it is framed by Capsule.

13. Example Layout

A Capsule file with:

- Version = 0001
- Encoding = A
- Header Length = 000A

would begin with this 16-byte prelude:

CAPSULE0001A000A

The next 10 bytes would be the Header Block.
All subsequent bytes would be the Payload Block.

## Header

### Header Lexical Structure for ASCII Encoding

When Encoding is A, the Header Block MUST consist of zero or more lines of
ASCII text.

A non-empty header field line MUST have the form:

key=value

Each line MUST be terminated by LF (%x0A).

The key MUST consist of one or more ASCII characters from the following set:

- A-Z
- a-z
- 0-9
- _
- -
- .

The first "=" character in a line MUST delimit the key from the value.

The value consists of all bytes following the first "=" up to but excluding
the terminating LF.

The value MAY be empty.

Empty lines MAY appear and MAY be ignored.

Header field names are case-sensitive unless otherwise specified by a
higher-level profile.

The meaning of header fields is application-defined unless otherwise
specified by a higher-level profile.

### ASCII Header Parsing Rules

A parser reading a Header Block with Encoding = A MUST:

1. split the Header Block into lines using LF (%x0A);
2. parse each non-empty line using the first "=" as the separator;
3. reject any non-empty line that does not contain "=";
4. reject any non-empty line whose key is empty;
5. MAY ignore unknown keys; and
6. SHOULD treat duplicate keys as invalid unless a higher-level profile defines duplicate-key behavior.


## Non-Normative Suggested Header Fields

The following header fields are suggested for general use. These fields are
application-defined and are not required by the Capsule base format unless
specified by a higher-level profile.

### dialect

The `dialect` field identifies the application-level dialect, schema, or
profile used to interpret the header and payload.

Example:

dialect=crispc/1.0.0

### encoding

The `encoding` field identifies the application-defined encoding or format of
the payload.

This field is distinct from the Capsule prelude Encoding field. The Capsule
prelude Encoding field specifies how the Header Block and Payload Block are
represented in the Capsule container. The header field `encoding` specifies
how the payload is to be interpreted by the application.

Example values:

encoding=cbor
encoding=bin
encoding=base64

### id

The `id` field provides an application-defined identifier for the payload.

### sig

The `sig` field contains a cryptographic signature associated with the
payload.

The signature format is application-defined unless specified by a higher-level
profile.

### key

The `key` field identifies the key associated with `sig`.

The value format is application-defined.

### encrypted

The `encrypted` field indicates whether the payload is encrypted.

Suggested values are:

encrypted=true
encrypted=false

### algorithm

The `algorithm` field identifies the algorithm associated with encryption,
signature, or related cryptographic processing.

The value format is application-defined.

Example values:

algorithm=AES-256-GCM
algorithm=Ed25519

### issuer

The `issuer` field identifies the issuer, authority, origin, or related
identifier associated with the payload, signature, or key material.

The value format is application-defined.
