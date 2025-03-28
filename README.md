# include::c!

Add C code to your Rust project within the same source file.

> **Warning**
Don't actually use this. It's just a quick hack I threw together.

## Example

```rust
use core::ffi::{c_char, CStr};

fn main() {
    let p: *const c_char = unsafe { hello() };
    let msg = unsafe { CStr::from_ptr(p) }.to_str().unwrap();
    println!("{msg}")
}

::include::c!(
    r#"
    const char *hello() {
        return "hello from c";
    }
"#
);
```

## How it works?

Instead of compiling the C code in it's own compilation unit and linking the file to
the rust project like the [cc](https://crates.io/crates/cc) crate, `include::c!` makes use
of a hacked up version of [c2rust](https://github.com/immunant/c2rust) to transpile the C code
into rust and inject it within a proc macro. This approach pulls in ~140 crates, but
who cares! You can write C in Rust.

The development experience is surprisingly good. You don't have to write FFI headers because you're just calling
Rust code . Rust-analyzer even works well enough. For example I got this error message while writing the example:

```rust
::include::c!(     ● similarly named function `baz` defined here
    r#"
    #include <stdlib.h>

    char* baz() {
        return malloc(7);
    }
"#
);

fn main() {
    println!("Hello, world!");
    let _x = unsafe { bar() };     ●● cannot find function `bar` in this scope
}
```

## Setup
```bash
# Follow the C2Rust installation steps for your system: <https://github.com/immunant/c2rust#installation> 
sudo apt install build-essential llvm clang libclang-dev cmake libssl-dev pkg-config python3 git

# Sorry, currently only works in rust 2021 projects
cargo new --edition 2021 example

cd example
cargo add include@0.1.0

# depends on your system, c2rust may default to the wrong llvm toolchain and fail to compile
export LLVM_CONFIG_PATH=llvm-config-14
```

## Syntax highlighting setup

You can configure neovim to syntax highlight the included C code.

```
; extends
;
; place in
; ~/.config/nvim/after/queries/rust/injections.scm
;
; this file must begin with ;extends, although it's in a comment, it's a directive to treesitter
;
; :Inspect to show the highlight groups under the cursor
; :InspectTree to show the parsed syntax tree ("TSPlayground")
; :EditQuery to open the Live Query Editor (Nvim 0.10+)

; https://www.youtube.com/watch?v=v3o9YaHBM4Q
; https://github.com/nvim-treesitter/nvim-treesitter?tab=readme-ov-file#adding-queries
;

; syntax highlight raw string inside ::include::c! as c code
(macro_invocation
  macro: (scoped_identifier path: (scoped_identifier name: (identifier) @_path (#eq? @_path "include") ) name: (identifier)  @_name (#eq? @_name "c"))
  (token_tree
    (raw_string_literal (string_content) @injection.content))
    (#set! injection.language "c"))
```

