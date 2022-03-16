# Querylizer

### v0.2.0 (2022-03-16)

- Backwards-incompatible!
- Add `deepform` for bodies of mixed Form and DeepObject parameters
- Pass encoding function as a reference to support `deepform` sharing it with `deepObject`.

### v0.1.4 (2022-03-15)

- Follow OpenAPI spec for query parameter encoding.
- Support x-www-form-urlencoded encoding.

### v0.1.3 (2022-03-03)

- Support `boolean` serialization.
- Always encode `+` in query to avoid historical ambiguity.

### v0.1.2 (2022-03-03)

- Backwards-incompatible!
- Pass extended string as mutable reference.
- Rename `escape_*` to `encode_*`.

### v0.1.1 (2022-03-03)

- Backwards-incompatible!
- Provide `name` parameter as string slice instead of Cow.

### v0.1.0 (2022-03-02)

- Support `deepObject`, `form`, and `simple` serialization.
