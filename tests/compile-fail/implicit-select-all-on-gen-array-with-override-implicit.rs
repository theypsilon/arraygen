#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(fn my_array: f32, implicit_select_all { override_implicit }: f32)] //~ERROR 8:31: 8:50: gen_array method 'my_array' contains implicit_select_all clause with forbidden decorator 'override_implicit'
struct Implicit {
    #[in_array(my_array)] 
    pub value: f32,
}
