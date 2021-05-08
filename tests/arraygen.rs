extern crate arraygen;

#[allow(non_snake_case)]
mod tests {
    use arraygen::Arraygen;

    #[test]
    fn test_arraygen___with_proper_declarations_and_includes___generates_correct_length_methods() {
        #[derive(Arraygen)]
        #[gen_array(fn booleans: &bool)]
        struct Test<'a> {
            #[in_array(booleans)]
            first: &'a bool,

            #[in_array(booleans)]
            second: &'a mut bool,

            #[in_array(booleans)]
            third: bool,
        }

        let foo = true;
        let mut bar = true;
        let actual = Test {
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
        struct Test {
            #[in_array(my_bools)]
            first: bool,

            #[in_array(my_bools)]
            #[in_array(booleans)]
            second: bool,
        }

        let actual = Test {
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
        struct Test {
            #[in_array(my_bools)]
            first: bool,

            #[in_array(booleans, my_bools)]
            second: bool,
        }

        let actual = Test {
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
        struct Test {
            #[in_array(trait_c)]
            a: A,

            #[in_array(trait_c)]
            b: B,
        }

        let actual = Test { a: A {}, b: B {} };
        assert_eq!(actual.trait_c().len(), 2);
    }

    #[test]
    fn test_arraygen___with_proper_declarations_and_includes___can_return_generic_types() {
        #[derive(Arraygen)]
        #[gen_array(fn some: Option<i32>)]
        struct Test {
            #[in_array(some)]
            a: Option<i32>,
        }

        let actual = Test { a: None };
        assert_eq!(actual.some().len(), 1);
    }

    #[test]
    fn test_arraygen___with_proper_declarations_and_includes___can_return_fn_types() {
        #[derive(Arraygen)]
        #[gen_array(fn functions: &fn() -> () )]
        struct Test {
            #[in_array(functions)]
            a: fn() -> (),
        }

        fn foo() {}

        let actual = Test { a: foo };
        assert_eq!(actual.functions().len(), 1);
    }

    #[test]
    fn test_arraygen___without_includes___generates_0_length_arrays() {
        #[derive(Arraygen)]
        #[gen_array(fn foo: i32)]
        struct Test {
            bar: i32,
        }

        let test = Test { bar: 2 };
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

        let mut sut = DisplaySimBug {
            rgb_red_r: RgbRedR { value: 3.0 },
        };

        assert_eq!(sut.get_ui_controllers_mut().len(), 1);
        assert_eq!(sut.get_ui_controllers().len(), 1);
    }
}
