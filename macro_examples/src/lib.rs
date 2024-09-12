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

/// Extracting enum variant content.
///
/// Helpful links:
/// https://veykril.github.io/tlborm/decl-macros/patterns/internal-rules.html
/// https://veykril.github.io/tlborm/syntax-extensions/source-analysis.html#token-trees
macro_rules! inner_as_mut {
    // Special matcher to reinterpret something as pattern.
    // Cannot be matched from the "outside" since `@` is not a valid character for our macro elements.
    (@as_pat $p:pat) => { $p };

    // Try to match value against the token tree reinterpreted as containing one value.
    // Will expand to something like this:
    // ```
    // if let Number::Integer(ref mut inner) = n1 {
    //     Some(inner)
    // } else {
    //     None
    // }
    // ```
    //
    // With `$($tts:tt)*`, we match repetitions of a token tree, so e.g. `Number::Integer` as
    // the tokens `Number` and `Integer`.
    // Pasting the repetitions of token trees (so `Number::Integer`) and concatenating it with
    // `(ref mut inner)` would give us a valid pattern.
    // However since the individual parts are not valid, this would not be parsed correctly.
    // Therefore, we pass the concatenation through a special case (of this same macro!) to
    // reinterpret it as pattern.
    ($value:expr, $($tts:tt)*) => {
        if let inner_as_mut!(@as_pat $($tts)*(ref mut inner)) = $value {
            Some(inner)
        } else {
            None
        }
    };
}

#[test]
fn use_inner_as_mut() {
    enum Number {
        Integer(i32),
        Float(f64),
    }

    let mut n1 = Number::Integer(1);
    let mut n2 = Number::Float(3.);

    let mutable_ref_n1 = inner_as_mut!(n1, Number::Integer);
    let mutable_ref_n2 = inner_as_mut!(n2, Number::Float);

    assert!(mutable_ref_n1.is_some());
    assert!(mutable_ref_n2.is_some());
}
