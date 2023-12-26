# Conerror

Provides a macro that automatically adds context to errors

# example
```rust
use conerror::conerror;
use std::fs::read;

fn main() {
    if let Err(e) = foo() {
        println!("{}", e);
    }
}

#[conerror]
fn foo() -> conerror::Result<()> {
    bar()?;
    Ok(())
}

#[conerror]
fn bar() -> conerror::Result<()> {
    baz()?;
    Ok(())
}

#[conerror]
fn baz() -> conerror::Result<()> {
    read("/root")?;
    Ok(())
}

```

Output:
```
Permission denied (os error 13)
src/main.rs:24 baz()
src/main.rs:18 bar()
src/main.rs:12 foo()
```