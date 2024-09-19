# Conerror

`conerror` is a Rust library designed to automatically add context to errors, 
making it easier to trace and debug issues by including file names, line numbers, 
and function names in error messages.

## Features

- Automatically adds context to errors.
- Works with any error type that implements std::error::Error.
- Provides detailed error tracebacks.

## Examples

Here's a basic example demonstrating how to use the conerror macro to add context to errors:

```rust
use conerror::conerror;
use std::fs::read;

fn main() {
    if let Err(e) = func1() {
        println!("{}", e);
    }
}

#[conerror]
fn func1() -> conerror::Result<()> {
    func2()?;
    Ok(())
}

#[conerror]
fn func2() -> conerror::Result<()> {
    Read.read()?;
    Ok(())
}

struct Read;

#[conerror]
impl Read {
    #[conerror]
    fn read(&self) -> conerror::Result<()> {
        read("/root")?;
        Ok(())
    }
}
```

### Output:

When the above example is run, it produces the following output:

```
Permission denied (os error 13)
#0 src/main.rs:28 untitled::Read::read()
#1 src/main.rs:18 untitled::func2()
#2 src/main.rs:12 untitled::func1()
```