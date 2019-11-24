extern crate arraygen;

#[allow(non_snake_case)]
mod tests {
    use arraygen::Arraygen;

    #[test]
    fn test_arraygen___generates_correct_length_methods() {
        #[derive(Arraygen)]
        #[gen_array(pub(crate) fn my_bools: &bool)]
        #[gen_array(pub fn poorly_named: &bool)]
        #[gen_array(fn booleans: &bool)]
        #[gen_array(fn bools_mut: &mut bool)]
        struct Test<'a> {
            #[in_array(my_bools)]
            #[in_array(poorly_named)]
            first: &'a bool,

            #[in_array(my_bools)]
            #[in_array(booleans)]
            second: &'a mut bool,

            #[in_array(bools_mut)]
            #[in_array(booleans)]
            third: bool,

            #[in_array(booleans)]
            fourth: &'a bool,
        }

        let foo = true;
        let mut bar = true;
        let mut actual = Test {
            first: &foo,
            second: &mut bar,
            third: true,
            fourth: &foo,
        };

        assert_eq!(actual.booleans().len(), 3);
        assert_eq!(actual.poorly_named().len(), 1);
        assert_eq!(actual.my_bools().len(), 2);
        assert_eq!(actual.bools_mut().len(), 1);
    }

    #[test]
    fn test_arraygen___can_return_trait_objects() {
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
    fn test_arraygen___can_return_generic_types() {
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
    fn test_arraygen___can_return_fn_types() {
        #[derive(Arraygen)]
        #[gen_array(fn functions: &fn() -> () )]
        struct Test {
            #[in_array(functions)]
            a: fn() -> ()
        }

        fn foo () {}

        let actual = Test { a: foo };
        assert_eq!(actual.functions().len(), 1);
    }
}
