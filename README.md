# querylizer

Rust serde library for OpenAPI v3 parameter styles.

Use `serde` to provide the different styles supported in OpenAPI parameters.

OpenAPI provides multiple styles for operation parameters.  `querylizer`
provides a serde Serializer for each style.

Currently supported styles are:
- `deepObject`
- `form`
- `simple`

There is also a `deepform` style that is useful for OpenAPI request bodies that mix `form` and `deepObject` styles.
