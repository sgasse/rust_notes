use std::ffi::{c_char, c_float, c_int, CString};

#[no_mangle]
extern "C" fn meow() {
    println!("Meow!");
}

// `c_int` matches `int` in C.
#[no_mangle]
extern "C" fn pass_cint_to_rust(arg: c_int) {
    println!("Received c_int {arg:?} from C");
}

// `i32` matches `int32_t` in C from `stdint.h`.
#[no_mangle]
extern "C" fn pass_int32_to_rust(arg: i32) {
    println!("Received i32 {arg} from C");
}

#[no_mangle]
extern "C" fn get_cint_from_rust() -> c_int {
    56
}

#[repr(C)]
pub struct Point {
    pub x: c_int,
    pub y: c_int,
}

#[no_mangle]
extern "C" fn get_point(x: c_int, y: c_int) -> *mut Point {
    // Allocated on the heap with `Box` for a stable address and return as pointer.
    Box::into_raw(Box::new(Point { x, y }))
}

// Heterogenous enums in Rust look easy on the Rust side and are more involved on the C side.
#[repr(C)]
pub enum Number {
    Integer(c_int),
    Float(c_float),
}

#[no_mangle]
extern "C" fn get_integer_number(x: c_int) -> *mut Number {
    Box::into_raw(Box::new(Number::Integer(x)))
}

#[no_mangle]
extern "C" fn get_float_number(x: c_float) -> *mut Number {
    Box::into_raw(Box::new(Number::Float(x)))
}

#[repr(C)]
pub struct NamedCollection {
    // Represent a `String`.
    pub name: *const c_char,
    // Represent a `Vec<i32>`.
    pub values_ptr: *mut i32,
    pub values_len: usize,
}

impl Drop for NamedCollection {
    fn drop(&mut self) {
        // Recreate CString to drop.
        drop(unsafe { CString::from_raw(self.name as *mut c_char) });

        // Recreate vector to drop.
        drop(unsafe { Vec::from_raw_parts(self.values_ptr, self.values_len, self.values_len) });

        println!("Deallocated NamedCollection in Rust");
    }
}

#[no_mangle]
extern "C" fn get_named_collection() -> *mut NamedCollection {
    let name = String::from("Primes");

    let mut values = vec![5, 6, 7];
    values.shrink_to_fit();

    assert_eq!(values.len(), values.capacity());
    let values_len = values.len();
    let values_ptr = values.as_mut_ptr();

    std::mem::forget(values);

    let collection = NamedCollection {
        name: CString::new(name)
            .expect("failed to create CString")
            .into_raw(),
        values_ptr,
        values_len,
    };

    Box::into_raw(Box::new(collection))
}

#[no_mangle]
extern "C" fn free_named_collection(ptr: *mut NamedCollection) {
    if ptr.is_null() {
        return;
    }
    // Recreate and drop with custom `Drop` implementation of `NamedCollection`.
    drop(unsafe { Box::<NamedCollection>::from_raw(ptr) });
}
