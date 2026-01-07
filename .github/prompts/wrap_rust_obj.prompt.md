---
agent: "agent"
model: Claude Haiku 4.5
name: wrap-obj
description: "Wrap a Rust-based object into Python using PyO3"
---

Make or finish making a Python wrapper for the ${input:rustObject} object using `PyO3`.

Requirements for the wrapper:

- Async functions should use `tokio` blocking to produce non-async python functions
-
