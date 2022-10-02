# Notes on `Send` and `Sync`

Some notes on the `Send` and `Sync` traits.

### Gist

- In a single-threaded application, Rust's ownership model (exclusive mutable
  borrows, lifetimes etc.) are enough to ensure memory safety and avoid data
  races.
- `Send` and `Sync` are specifically targeted at indicating which guarantees for
  memory safety we have in a multi-threaded context.
- All primitive types are `Send` and `Sync` and any type composed only of
  `Send`/`Sync` members is itself `Send`/`Sync`.
- `Send` means it is safe to transfer the ownership of a value to another
  thread.
  - To understand why this is even needed, we have to take a look at types which
    are **not** `Send`. Not being `Send` means that we could run into undefined
    behavior if we were able to transfer such a type to another thread.
    - The prime example is `Rc<T>`, a reference-counted value. `Rc<T>` can be
      freely cloned within a thread. `unsafe` code behind safe wrappers ensures
      that the memory is only freed once the last reference went out of scope,
      which means that the reference count went to zero.
    - `Rc<T>` is optimized for single-thread usage, the reference count is not
      atomic. Very roughly, this means that reading and updating it cannot
      happen in one operation.
    - Imagine two thread having multiple clones of a `Rc<T>`, thread one has 1
      and thread two has 3. The total count is 4. When the one clone in thread
      one goes out of scope, it might read the total count of 4 and want to
      store 3. However between the read of 4 and storing 3, thread two might
      add another clone, raising the total count to 5, which could be
      overwritten by the 3 of thread one. If this was allowed, the memory
      could be freed when thread two still holds one clone of the `Rc<T>`
      and accessing this memory would lead to undefined behavior.
- `Sync` means it is safe to reference a value from several threads.
  - The exact definition is therefore even "A type `T` is `Sync` if `&T` is
    `Send`.
  - `RefCell<T>` allows us to do borrow-checking at runtime, but the underlying
    implementation is optimized for the single-thread use-case and not safe to
    be used from multiple threads. `RefCell<T>` is `Send` (if `T` is `Send`),
    but `&RefCell<T>` is not `Send`, thus `RefCell<T>` is not `Sync`.
- `Arc` is the multi-threaded alternative to `Rc` - it is thread-safe but
  relies on atomics, which would be wasteful to use in the single-threaded
  context where they are not needed.
- `Arc<T>` cannot make any type `Send`/`Sync` that was not `Send`/`Sync` before.
- `Mutex<T>` provides thread-safe borrow-checking at runtime using atomics.
  For a type `T` that is `Send`, `Mutex<T>` is `Sync`.

### Resources

- [Book Chapter][send_sync_book]
- [`Send` and `Sync` in the Nomicon][send_sync_nomicon]

[send_sync_book]: https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html
[send_sync_nomicon]: https://doc.rust-lang.org/nomicon/send-and-sync.html
