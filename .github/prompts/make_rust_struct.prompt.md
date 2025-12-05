---
agent: "agent"
model: Claude Haiku 4.5
name: make-struct
description: "Generate a Rust struct to deserialize a given file using Serde"
---

Create a Rust struct for deserializing this file using the `Serde` crate. Only give me the necessary structs, no examples, `use` statements, or Cargo modifications are needed.

Requirements for the structs:

- Include the `Debug` and `Clone` derive implementations for all structs
- Use serde `rename_all` if multiple fields need to be renamed
-
