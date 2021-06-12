#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(fn my_array: Option<f32>, implicit_select_all: _, Option<_>)] //~ERROR 8:63: 8:72: gen_array method 'my_array' contains implicit_select_all clause with duplicated 'Option < _ >' type
struct Implicit1 {
    pub value: Option<f32>,
}

#[derive(Arraygen)]
#[gen_array(fn my_array: Option<f32>, implicit_select_all: Option<_>, _)] //~ERROR 14:71: 14:72: gen_array method 'my_array' contains implicit_select_all clause with duplicated '_' type
struct Implicit2 {
    pub value: Option<f32>,
}

#[derive(Arraygen)]
#[gen_array(fn my_array: Option<f32>, implicit_select_all: _, Option<f32>)] //~ERROR 20:63: 20:74: gen_array method 'my_array' contains implicit_select_all clause with duplicated 'Option < f32 >' type
struct Implicit3 {
    pub value: Option<f32>,
}

#[derive(Arraygen)]
#[gen_array(fn my_array: Option<f32>, implicit_select_all:  Option<f32>, _)] //~ERROR 26:74: 26:75: gen_array method 'my_array' contains implicit_select_all clause with duplicated '_' type
struct Implicit4 {
    pub value: Option<f32>,
}

#[derive(Arraygen)]
#[gen_array(fn my_array: Option<f32>, implicit_select_all: Option<_>, Option<f32>)] //~ERROR 32:71: 32:82: gen_array method 'my_array' contains implicit_select_all clause with duplicated 'Option < f32 >' type
struct Implicit5 {
    pub value: Option<f32>,
}

#[derive(Arraygen)]
#[gen_array(fn my_array: Option<f32>, implicit_select_all:  Option<f32>, Option<_>)] //~ERROR 38:74: 38:83: gen_array method 'my_array' contains implicit_select_all clause with duplicated 'Option < _ >' type
struct Implicit6 {
    pub value: Option<f32>,
}