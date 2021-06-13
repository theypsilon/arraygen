extern crate arraygen;

#[allow(non_snake_case)]
mod tests {
    use arraygen::Arraygen;

    #[test]
    fn test_trait_objects_return_type___with_proper_declarations_and_includes___compiles_as_expected() {
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
    fn test_trait_objects_return_type___with_display_sim_bug_scenario___compiles_as_expected() {
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
}
