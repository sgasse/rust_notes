use std::collections::HashMap;

macro_rules! updateable_struct {
    // Optimization for one field
    ($name:ident, [$field_name:ident, $id:pat] ) => {
        #[derive(Debug, Default)]
        pub struct $name {
                $field_name: String,
        }
        impl $name {
            pub fn maybe_update(&mut self, data: HashMap<u64, String>) {
                if let Some(val) = data.get($id) {
                    self.$field_name = val.clone();
                }
            }
        }
    };
    // Case for several fields
    ($name:ident, $( [$field_name:ident, $id:pat] )+) => {
        #[derive(Debug, Default)]
        pub struct $name {
            $(
                $field_name: String,
            )+
        }
        impl $name {
            pub fn maybe_update(&mut self, data: HashMap<u64, String>) {
                for (key, val) in data {
                    match key {
                        $(
                            $id => {
                                self.$field_name = val.clone();
                            }
                        )+
                        _ => {}
                    }
                }
            }
        }
    };
}

macro_rules! struct_wrapper {
    // $(...)* repeats zero or more times
    // $(...)+ repeats one or more times
    // $(...);+ has a semicolon between repetitions
    // $foo:ident names an identifier, a type name in this example
    // $foo:pat names a pattern, we use it in a `match` expression here
    // $foo:expr names an expression which can be used in different places
    ( $($wrapper_field_name:ident, $struct_name:ident, $id:pat, [$( $field_name:ident, $field_id:pat ),+ ]);+ ) => {
        $(
            updateable_struct!(
                $struct_name,
                $(
                    [
                        $field_name,
                        $field_id
                    ]
                )+
            );
        )+

        #[derive(Debug, Default)]
        pub struct MainWrapper {
            $(
                $wrapper_field_name: $struct_name,
            )+
        }
        impl MainWrapper {
            #[allow(dead_code)]
            fn maybe_update(&mut self, id: u64, update: HashMap<u64, String>) {
                match id {
                    $(
                        $id => self.$wrapper_field_name.maybe_update(update),
                    )+
                    _ => {},
                }
            }
        }

    };
}

// To see the expanded macro in VS code, over the macro call, Ctrl+Shift+P and
// look for `rust-analyzer: Expand macro recursively`.
struct_wrapper!(
    main_field1, MyStruct, 200,
    [
        subfield1, 10,
        subfield2, 20
    ];
    main_field2, MySecondStruct, 300,
    [
        subfield3, 30,
        subfield4, 40
    ]
);
