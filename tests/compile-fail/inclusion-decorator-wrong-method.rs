#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array: i32)]
struct Test{
    #[in_array(my_array2 { cast })] //~ERROR 10:16: 10:25: gen_array method 'my_array2' not present but used by field 'foo'
    foo: i32
}