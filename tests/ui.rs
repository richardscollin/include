#![allow(missing_unsafe_on_extern)]

::include::c!(
    r#"
    int add(int a, int b) {
        return a + b;
    }
"#
);

#[test]
fn check_add() {
    let x = unsafe { add(1, 2) };
    assert_eq!(x, 3);
}

::include::c!(
    r#"
    const char *hello() {
        return "hello from c";
    }
"#
);

use core::ffi::{c_char, CStr};

#[test]
fn check_get() {
    let p: *const c_char = unsafe { hello() };
    let msg = unsafe { CStr::from_ptr(p) }.to_str().unwrap();
    println!("{msg}")
}

::include::c!(
    r#"
    #include <stdlib.h>

    char* baz() {
        return malloc(7);
    }
"#
);

fn main() {
    println!("Hello, world!");
    let _x = unsafe { baz() };
}
