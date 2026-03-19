# Encrypted File Format v1

## Header

- Magic: `GSC1` (4 bytes)
- Version: `1` (1 byte)
- Algorithm ID: `1` for `AesSivV1` (1 byte)

Header length is 6 bytes total.

## Payload

- Ciphertext produced by deterministic AES-SIV encryption backend.
- Associated data is repository-relative path bytes.

## Parsing rules

- Magic mismatch -> not encrypted in this format.
- Unknown version/algorithm -> parse failure.
- Decrypt failure -> integrity/authentication failure.

## Compatibility

- New algorithms must add a new algorithm ID.
- Version upgrades must preserve backward parsing where feasible.
