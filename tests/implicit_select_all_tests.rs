extern crate arraygen;

#[allow(non_snake_case)]
mod tests {
    use arraygen::Arraygen;

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

        let actual = Sut { a: Ok(1), b: 3.0 };

        assert_eq!(actual.options().len(), 1);
    }

}
