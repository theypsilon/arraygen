#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(fn my_array: f32, implicit_select_all: f32)]
struct Implicit {
    #[in_array(my_array)] //~ERROR 10:16: 10:24: Field 'value' is already included in gen_array method 'my_array'
    pub value: f32,
}
