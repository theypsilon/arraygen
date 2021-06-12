#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(fn my_array: f32, implicit_select_all: _, _)] //~ERROR 8:52: 8:53: gen_array method 'my_array' contains implicit_select_all clause with duplicated '_' type
struct Implicit {
    pub value: f32,
}
