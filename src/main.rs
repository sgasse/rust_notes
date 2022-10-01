use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

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

// trait IsStatic {
//     fn check_static(&self);
// }

// impl<T> IsStatic for DispatchWrapper<T>
// where
//     T: 'static,
// {
//     fn check_static(&self) {
//         println!("{} has 'static bound", std::any::type_name::<T>());
//     }
// }

// trait IsNotStatic {
//     fn check_static(&self);
// }

// impl<T> IsNotStatic for &T {
//     fn check_static(&self) {
//         println!("{} does not have 'static bound", std::any::type_name::<T>());
//     }
// }

// macro_rules! check_if_static {
//     ($e:expr) => {
//         let wrapper = DispatchWrapper { _data: $e };
//         (&wrapper).check_static();
//     };
// }

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
    let mutex = Mutex::new(1);

    check_values!(
        23_i32,
        &23_i32,
        Rc::new(1),
        &Rc::new(1),
        RefCell::new(1),
        refcell.borrow(),
        Arc::new(1),
        &Arc::new(1),
        Mutex::new(1),
        mutex.lock()
    );
}

fn main() {
    print_threading_trait_impls();
}
