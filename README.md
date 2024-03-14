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
    Baz.baz()?;
    Ok(())
}

struct Baz;

#[conerror]
impl Baz {
    #[conerror]
    fn baz(&self) -> conerror::Result<()> {
        read("/root")?;
        Ok(())
    }
}
```

Output:

```
Permission denied (os error 13)
#0 src/main.rs:28 demo::Baz::baz()
#1 src/main.rs:18 demo::bar()
#2 src/main.rs:12 demo::foo()
```