extern crate arraygen;

#[allow(dead_code)]
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

        assert_eq!(prices.get_all_prices().into_iter().sum::<f32>(), 14.0);
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

    #[test]
    fn test_implicit_select_all___with_display_sim_bug_scenario___compiles_as_expected() {
        impl<T> ResetOption for Option<T> {
            fn reset(&mut self) {
                *self = None;
            }
        }

        pub trait ResetOption {
            fn reset(&mut self);
        }

        pub trait Tracked {}
        impl Tracked for i32 {}
        impl Tracked for Result<i32, i32> {}

        #[derive(Arraygen)]
        #[gen_array(pub fn options: &mut dyn ResetOption, implicit_select_all: Option<_>)]
        #[gen_array(pub fn tracked: &mut dyn Tracked, implicit_select_all: i32, Result<i32, i32>)]
        pub struct Sut {
            // tracked
            pub a: Result<i32, i32>,
            pub b: i32,

            // options
            pub c: Option<i32>,
        }

        let mut actual = Sut {
            a: Ok(1),
            b: 3,
            c: Some(1),
        };
        assert_eq!(actual.options().len(), 1);
        assert_eq!(actual.tracked().len(), 2);
    }
}
