extern crate arraygen;

#[allow(non_snake_case)]
mod tests {
    use arraygen::Arraygen;

    #[test]
    fn test_arraygen___with_proper_declarations_and_includes___generates_correct_length_methods() {
        #[derive(Arraygen)]
        #[gen_array(fn booleans: &bool)]
        struct Sut<'a> {
            #[in_array(booleans)]
            first: &'a bool,

            #[in_array(booleans)]
            second: &'a mut bool,

            #[in_array(booleans)]
            third: bool,
        }

        let foo = true;
        let mut bar = true;
        let actual = Sut {
            first: &foo,
            second: &mut bar,
            third: true,
        };

        assert_eq!(actual.booleans().len(), 3);
    }

    #[test]
    fn test_arraygen___with_proper_declarations_and_duplicated_includes___generates_correct_length_methods(
    ) {
        #[derive(Arraygen)]
        #[gen_array(pub(crate) fn my_bools: &bool)]
        #[gen_array(fn booleans: bool)]
        struct Sut {
            #[in_array(my_bools)]
            first: bool,

            #[in_array(my_bools)]
            #[in_array(booleans)]
            second: bool,
        }

        let actual = Sut {
            first: true,
            second: false,
        };

        assert_eq!(actual.booleans().len(), 1);
        assert_eq!(actual.my_bools().len(), 2);
    }

    #[test]
    fn test_arraygen___with_proper_declarations_and_multi_includes___generates_correct_length_methods(
    ) {
        #[derive(Arraygen)]
        #[gen_array(pub(crate) fn my_bools: &bool)]
        #[gen_array(fn booleans: bool)]
        struct Sut {
            #[in_array(my_bools)]
            first: bool,

            #[in_array(booleans, my_bools)]
            second: bool,
        }

        let actual = Sut {
            first: true,
            second: false,
        };

        assert_eq!(actual.booleans().len(), 1);
        assert_eq!(actual.my_bools().len(), 2);
    }

    #[test]
    fn test_arraygen___with_proper_declarations_and_includes___can_return_trait_objects() {
        struct A {}
        struct B {}
        trait C {}
        impl C for A {}
        impl C for B {}

        #[derive(Arraygen)]
        #[gen_array(fn trait_c: &dyn C)]
        struct Sut {
            #[in_array(trait_c)]
            a: A,

            #[in_array(trait_c)]
            b: B,
        }

        let actual = Sut { a: A {}, b: B {} };
        assert_eq!(actual.trait_c().len(), 2);
    }

    #[test]
    fn test_arraygen___with_proper_declarations_and_includes___can_return_generic_types() {
        #[derive(Arraygen)]
        #[gen_array(fn some: Option<i32>)]
        struct Sut {
            #[in_array(some)]
            a: Option<i32>,
        }

        let actual = Sut { a: None };
        assert_eq!(actual.some().len(), 1);
    }

    #[test]
    fn test_arraygen___with_proper_declarations_and_includes___can_return_fn_types() {
        #[derive(Arraygen)]
        #[gen_array(fn functions: &fn() -> () )]
        struct Sut {
            #[in_array(functions)]
            a: fn() -> (),
        }

        fn foo() {}

        let actual = Sut { a: foo };
        assert_eq!(actual.functions().len(), 1);
    }

    #[test]
    fn test_arraygen___without_includes___generates_0_length_arrays() {
        #[derive(Arraygen)]
        #[gen_array(fn foo: i32)]
        struct Sut {
            bar: i32,
        }

        let test = Sut { bar: 2 };
        let _ = test.bar; // Avoiding warning

        assert_eq!(test.foo().len(), 0);
    }

    #[test]
    fn test_arraygen___display_sim___bug_is_fixed() {
        trait UiController {
            fn give(&self) -> f32;
        }

        #[derive(Clone)]
        struct RgbRedR {
            pub value: f32,
        }
        impl UiController for RgbRedR {
            fn give(&self) -> f32 {
                self.value
            }
        }

        #[derive(Clone, Arraygen)]
        #[gen_array(pub fn get_ui_controllers: &dyn UiController)]
        #[gen_array(pub fn get_ui_controllers_mut: &mut dyn UiController)]
        struct DisplaySimBug {
            #[in_array(get_ui_controllers, get_ui_controllers_mut)]
            rgb_red_r: RgbRedR,
        }

        let mut actual = DisplaySimBug {
            rgb_red_r: RgbRedR { value: 3.0 },
        };

        assert_eq!(actual.get_ui_controllers_mut().len(), 1);
        assert_eq!(actual.get_ui_controllers().len(), 1);
    }

    #[test]
    fn test_implicit_select_all___given_f32___returns_all_f32_fields() {
        #[derive(Arraygen)]
        #[gen_array(fn get_all_prices: f32, implicit_select_all: f32)]
        struct ImplicitPrices {
            pub water: f32,
            pub oil: f32,
            pub tomato: f32,
            pub chocolate: f32,
            pub gold: f64,
        }

        let prices = ImplicitPrices {
            water: 2.0,
            oil: 4.0,
            tomato: 3.0,
            chocolate: 5.0,
            gold: 1000.0,
        };

        assert_eq!(prices.get_all_prices().iter().sum::<f32>(), 14.0);
    }

    #[test]
    fn test_implicit_select_all___given_wildcard___returns_all_fields() {
        trait All {}

        struct A {}
        struct B {}
        struct C {}

        impl All for A {}
        impl All for B {}
        impl All for C {}

        #[derive(Arraygen)]
        #[gen_array(fn all: &dyn All, implicit_select_all: _)]
        struct Sut {
            pub a: A,
            pub b: B,
            pub c: C,
        }

        let actual = Sut {
            a: A {},
            b: B {},
            c: C {},
        };

        assert_eq!(actual.all().len(), 3);
    }

    #[test]
    fn test_implicit_select_all___given_option_with_wildcard___returns_all_option_fields() {
        trait OptionTrait {}

        impl<T> OptionTrait for Option<T> {}

        #[derive(Arraygen)]
        #[gen_array(fn options: &dyn OptionTrait, implicit_select_all: Option<_>)]
        struct Sut {
            pub a: Option<i32>,
            pub b: Option<bool>,
            pub c: f32,
        }

        let actual = Sut {
            a: Some(1),
            b: Some(false),
            c: 3.0,
        };

        assert_eq!(actual.options().len(), 2);
    }

    #[test]
    fn test_implicit_select_all___given_result_with_wildcard___returns_all_result_fields() {

        #[derive(Arraygen)]
        #[gen_array(fn options: &Result<i32, std::fmt::Error>, implicit_select_all: Result<_, std::fmt::Error>)]
        struct Sut {
            pub a: Result<i32, std::fmt::Error>,
            pub b: f32,
        }

        let actual = Sut {
            a: Ok(1),
            b: 3.0,
        };

        assert_eq!(actual.options().len(), 1);
    }

    #[test]
    fn test_in_array_cast___to_different_type___compiles_correctly() {
        #[derive(Arraygen)]
        #[gen_array(fn bytes: u8)]
        struct Sut {
            #[in_array(bytes { cast })]
            pub a: i32,
        }

        let actual = Sut { a: -1 };

        assert_eq!(actual.bytes()[0], 255);
    }

    #[test]
    fn test_in_array_unsafe_transmute___between_numbers___compiles_correctly() {
        #[derive(Arraygen)]
        #[gen_array(fn numbers: i32)]
        struct Sut {
            #[in_array(numbers { unsafe_transmute })]
            pub a: f32,
        }

        let actual = Sut { a: -1.23456789 };

        assert_eq!(actual.numbers()[0], -1080162734);
    }

    #[test]
    fn test_in_array_unsafe_transmute___between_refs___compiles_correctly() {
        #[derive(Arraygen)]
        #[gen_array(fn number_refs: &i32)]
        struct Sut<'a> {
            #[in_array(number_refs { unsafe_transmute })]
            pub a: &'a f32,
        }

        let foo = -1.23456789;

        let actual = Sut { a: &foo };

        assert_eq!(*actual.number_refs()[0], -1080162734);
    }

    #[test]
    fn test_in_array_unsafe_transmute___from_number_to_ref___compiles_correctly() {
        #[derive(Arraygen)]
        #[gen_array(fn number_refs: &i32)]
        struct Sut {
            #[in_array(number_refs { unsafe_transmute })]
            pub a: f32,
        }

        let actual = Sut { a: -1.23456789 };

        assert_eq!(*actual.number_refs()[0], -1080162734);
    }
}
