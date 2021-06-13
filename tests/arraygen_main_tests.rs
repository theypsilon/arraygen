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
}
