use std::{cell::RefCell, rc::Rc};

// Some inspiration
// // https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md

struct DispatchWrapper<T> {
    _data: T,
}

trait ImplSend {
    fn check_send(&self);
}

impl<T> ImplSend for DispatchWrapper<T>
where
    T: Send,
{
    fn check_send(&self) {
        println!("{} implements Send", std::any::type_name::<T>());
    }
}

trait NotImplSend {
    fn check_send(&self);
}

impl<T> NotImplSend for &T {
    fn check_send(&self) {
        println!("{} does not implement Send", std::any::type_name::<T>());
    }
}

macro_rules! check_if_send {
    ($e: expr) => {
        let wrapper = DispatchWrapper { _data: $e };
        (&wrapper).check_send();
    };
}

trait ImplSync {
    fn check_sync(&self);
}

impl<T> ImplSync for DispatchWrapper<T>
where
    T: Sync,
{
    fn check_sync(&self) {
        println!("{} implements Sync", std::any::type_name::<T>());
    }
}

trait NotImplSync {
    fn check_sync(&self);
}

impl<T> NotImplSync for &T {
    fn check_sync(&self) {
        println!("{} does not implement Sync", std::any::type_name::<T>());
    }
}

macro_rules! check_if_sync {
    ($e:expr) => {
        let wrapper = DispatchWrapper { _data: $e };
        (&wrapper).check_sync();
    };
}

macro_rules! check_values {
    ($($e:expr), *) => {
        $(
            check_if_send!($e);
            check_if_sync!($e);
        )*
    };
}

pub fn print_threading_trait_impls() {
    let refcell = RefCell::new(1);

    check_values!(
        23_i32,
        &23_i32,
        Rc::new(1),
        &Rc::new(1),
        RefCell::new(1),
        refcell.borrow()
    );
}

fn main() {
    print_threading_trait_impls();
}

fn take_any<T>(_value: T) {
    println!("Took value any {:?}", std::any::type_name::<T>());
}

fn take_send<T>(_value: T)
where
    T: Send,
{
    println!("Took value send {:?}", std::any::type_name::<T>());
}

fn take_sync<T>(_value: T)
where
    T: Sync,
{
    println!("Took value sync {:?}", std::any::type_name::<T>());
}

fn take_send_sync<T>(_value: T)
where
    T: Send + Sync,
{
    println!("Took value send+sync {:?}", std::any::type_name::<T>());
}

fn take_send_sync_static<T>(_value: T)
where
    T: Send + Sync + 'static,
{
    println!(
        "Took value send+sync+'static {:?}",
        std::any::type_name::<T>()
    );
}
