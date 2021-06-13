extern crate arraygen;

#[allow(non_snake_case)]
mod tests {
    use arraygen::Arraygen;

    #[test]
    fn test_casting_on_in_array___returns_expected_result() {
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
    fn test_unsafe_transmute_on_in_array___returns_expected_result() {
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
    fn test_unsafe_transmute_on_in_array___between_refs___returns_expected_result() {
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
    fn test_unsafe_transmute_on_in_array___from_number_to_ref___compiles_correctly() {
        #[derive(Arraygen)]
        #[gen_array(fn number_refs: &i32)]
        struct Sut {
            #[in_array(number_refs { unsafe_transmute })]
            pub a: f32,
        }

        let actual = Sut { a: -1.23456789 };

        assert_eq!(*actual.number_refs()[0], -1080162734);
    }

    #[test]
    fn test_casting_on_implicit_select_all___returns_expepcted_sum() {
        #[derive(Arraygen)]
        #[gen_array(fn number_refs: f64, implicit_select_all { cast }: i8, u8, i16, u16, i32, u32, f32, f64)]
        struct Sut {
            pub a: i32,
            pub b: u32,
            pub c: i16,
            pub d: u16,
            pub e: f32,
        }

        let actual = Sut {
            a: 1,
            b: 1,
            c: 1,
            d: 1,
            e: 1.0,
        };

        assert_eq!(actual.number_refs().iter().sum::<f64>(), 5.0);
    }

    #[test]
    fn test_overrideding_cast___from_implicit_select_all_with_no_decorator___compiles_correctly() {
        #[derive(Arraygen)]
        #[gen_array(fn my_array: f32, implicit_select_all: f32)]
        struct _Sut1 {
            #[in_array(my_array { override_implicit, cast })]
            pub a: i32,
        }

        #[derive(Arraygen)]
        #[gen_array(fn my_array: f32, implicit_select_all: f32)]
        struct _Sut2 {
            #[in_array(my_array { cast, override_implicit })]
            pub a: i32,
        }
    }
}
