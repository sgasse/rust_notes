use prettytable::{row, Table};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
struct SendSyncRes {
    name: String,
    send: bool,
    sync: bool,
}

/// Get the type name of a value passed in.
///
/// This is done in a "best-effort" manner following the compiler
/// infrastructure. Some names appear unnecessary long.
fn get_type_name<T>(_value: T) -> String {
    std::any::type_name::<T>().to_owned()
}

/// Wrap data of type `T` so that we can implement traits on the Wrapper.
///
/// For the autoref trick, we need to implement two traits which have the same
/// method name on a type where one trait implementation is more specific.
/// We wrap our data in a struct so that we can add a trait bound like `Send`
/// to the wrapped data and get a more specific bound.
struct DispatchWrapper<T> {
    _data: T,
}

/// Trait to indicate that the wrapped type implements Send.
trait ImplSend {
    fn is_send(&self) -> bool;
}

/// Implementation of check function for wrapper type.
///
/// With the trait bound on `Send`, this can only be dispatched if the wrapped
/// type `T` implements `Send`.
impl<T> ImplSend for DispatchWrapper<T>
where
    T: Send,
{
    fn is_send(&self) -> bool {
        true
    }
}

/// Generic fallback trait to indicate that the wrapped type does not implement Send.
trait NotImplSend {
    fn is_send(&self) -> bool;
}

/// Generic fallback implementation of check function.
///
/// Given that this does not have a trait bound, the autoref resolution will
/// dispatch to this trait's method if the more specific implementation of
/// `ImplSend` does not apply.
impl<T> NotImplSend for &T {
    fn is_send(&self) -> bool {
        false
    }
}

/// Check if the type of an expression is `Send`.
///
/// This relies on the autoref resolution trick where the compiler will resolve
/// the ambiguous method call to the trait which is implemented directly on `T`
/// and only falls back to the implementation of **another** trait implemented
/// on `&T` if this does not exist. This is a workaround for the still-missing
/// specialization feature in Rust:
/// https://github.com/rust-lang/rfcs/blob/master/text/1210-impl-specialization.md
/// Inspired by:
/// https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md
macro_rules! check_if_send {
    ($e: expr) => {{
        let data = $e;
        let wrapper = DispatchWrapper { _data: data };
        (&wrapper).is_send()
    }};
}

/// Trait to indicate that the wrapped type implements Sync.
trait ImplSync {
    fn is_sync(&self) -> bool;
}

/// Implementation of check function for wrapper type.
///
/// With the trait bound on `Sync`, this can only be dispatched if the wrapped
/// type `T` implements `Sync`.
impl<T> ImplSync for DispatchWrapper<T>
where
    T: Sync,
{
    fn is_sync(&self) -> bool {
        true
    }
}

/// Generic fallback trait to indicate that the wrapped type does not implement Sync.
trait NotImplSync {
    fn is_sync(&self) -> bool;
}

/// Generic fallback implementation of check function.
///
/// Given that this does not have a trait bound, the autoref resolution will
/// dispatch to this trait's method if the more specific implementation of
/// `ImplSync` does not apply.
impl<T> NotImplSync for &T {
    fn is_sync(&self) -> bool {
        false
    }
}

/// Check if the type of an expression is `Send`.
///
/// This relies on the autoref resolution trick where the compiler will resolve
/// the ambiguous method call to the trait which is implemented directly on `T`
/// and only falls back to the implementation of **another** trait implemented
/// on `&T` if this does not exist. This is a workaround for the still-missing
/// specialization feature in Rust:
/// https://github.com/rust-lang/rfcs/blob/master/text/1210-impl-specialization.md
/// Inspired by:
/// https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md
macro_rules! check_if_sync {
    ($e:expr) => {{
        let data = $e;
        let wrapper = DispatchWrapper { _data: data };
        (&wrapper).is_sync()
    }};
}

/// Check expressions for whether their underlying types are Send / Sync.
macro_rules! check_values {
    ($($e:expr), *) => {{
        let mut results: Vec<SendSyncRes> = Vec::new();
        $(
            let name = get_type_name($e);
            let send = check_if_send!($e);
            let sync = check_if_sync!($e);
            results.push(SendSyncRes { name, send, sync});
        )*

        results
    }
    };
}

/// Check various expressions for Send/Sync and print the result as table.
pub fn print_threading_trait_impls() {
    // Check different types for Send/Sync based on representative values
    let send_sync_results = check_values!(
        1_i32,
        &1_i32,
        Rc::new(1),
        &Rc::new(1),
        RefCell::new(1),
        &RefCell::new(1),
        Arc::new(1),
        &Arc::new(1),
        Arc::new(RefCell::new(1)),
        &Arc::new(RefCell::new(1)),
        Mutex::new(1),
        &Mutex::new(1),
        Arc::new(Mutex::new(RefCell::new(1))),
        Arc::new(Mutex::new(Rc::new(1)))
    );

    let mut table = Table::new();
    table.add_row(row!["Name", "Send?", "Sync?"]);
    for res in send_sync_results {
        table.add_row(row![res.name, res.send, res.sync]);
    }

    table.printstd();
}

fn main() {
    print_threading_trait_impls();
}
