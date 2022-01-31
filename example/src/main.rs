use arraygen::Arraygen;

fn main() {
    read_prices();
    to_lowercase();
    call_trait_objects();
    implicit_select_all();
    casts();
}

fn read_prices() {
    let prices = Prices {
        water: 1.0,
        oil: 3.0,
        tomato: 2.0,
        chocolate: 4.0,
    };

    println!(
        "Sum of all prices: {}",
        prices.get_all_prices().into_iter().sum::<f32>()
    );
}

#[derive(Arraygen)]
#[gen_array(fn get_all_prices: f32)]
struct Prices {
    #[in_array(get_all_prices)]
    pub water: f32,
    #[in_array(get_all_prices)]
    pub oil: f32,
    #[in_array(get_all_prices)]
    pub tomato: f32,
    #[in_array(get_all_prices)]
    pub chocolate: f32,
}

fn to_lowercase() {
    let mut person = Person {
        first_name: "ADa".into(),
        last_name: "LoVElaCE".into(),
    };

    for name in person.get_names().into_iter() {
        *name = name.to_lowercase();
    }

    println!("Lowercase Ada name is: {}", person);
}

#[derive(Arraygen)]
#[gen_array(fn get_names: &mut String)]
struct Person {
    #[in_array(get_names)]
    pub first_name: String,
    #[in_array(get_names)]
    pub last_name: String,
}

impl std::fmt::Display for Person {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{} {}", self.first_name, self.last_name)
    }
}

fn call_trait_objects() {
    let animals = Animals {
        dogo: Dog {},
        tiger: Cat {},
        bacon: Pig {},
    };

    let talk = animals
        .get_animals()
        .into_iter()
        .map(|animal| animal.talk())
        .collect::<Vec<&'static str>>()
        .join(", ");

    println!("Animals say: {}", talk);
}

trait Animal {
    fn talk(&self) -> &'static str;
}
struct Dog {}
impl Animal for Dog {
    fn talk(&self) -> &'static str {
        "bark"
    }
}
struct Cat {}
impl Animal for Cat {
    fn talk(&self) -> &'static str {
        "meow"
    }
}
struct Pig {}
impl Animal for Pig {
    fn talk(&self) -> &'static str {
        "oink"
    }
}

#[derive(Arraygen)]
#[gen_array(fn get_animals: &dyn Animal)]
struct Animals {
    #[in_array(get_animals)]
    dogo: Dog,
    #[in_array(get_animals)]
    tiger: Cat,
    #[in_array(get_animals)]
    bacon: Pig,
}

fn implicit_select_all() {
    implicit_select_all_prices();
    implicit_select_all_animals();
    implicit_select_all_with_wildcards();
}

fn implicit_select_all_prices() {
    let prices = ImplicitPrices {
        water: 2.0,
        oil: 4.0,
        tomato: 3.0,
        chocolate: 5.0,
    };

    println!(
        "Sum of all implicit prices: {}",
        prices.get_all_prices().into_iter().sum::<f32>()
    );
}

#[derive(Arraygen)]
#[gen_array(fn get_all_prices: f32, implicit_select_all: f32)]
struct ImplicitPrices {
    pub water: f32,
    pub oil: f32,
    pub tomato: f32,
    pub chocolate: f32,
}

fn implicit_select_all_animals() {
    let animals = ImplicitAnimals {
        dogo: Dog {},
        tiger: Cat {},
        bacon: Pig {},
    };

    let talk = animals
        .get_animals()
        .into_iter()
        .map(|animal| animal.talk())
        .collect::<Vec<&'static str>>()
        .join(", ");

    println!("Implicit animals say: {}", talk);
}

#[derive(Arraygen)]
#[gen_array(fn get_animals: &dyn Animal, implicit_select_all: Dog, Cat, Pig)]
struct ImplicitAnimals {
    bacon: Pig,
    dogo: Dog,
    tiger: Cat,
}

fn implicit_select_all_with_wildcards() {
    let mut options = Options {
        a: Some(1),
        b: Some(true),
        c: 3.0,
    };

    println!("Options before reset: {:?}", options);
    options.options().into_iter().for_each(|o| o.reset());
    println!("Options after reset: {:?}", options);
}

impl<T> ResetOption for Option<T> {
    fn reset(&mut self) {
        *self = None;
    }
}

trait ResetOption {
    fn reset(&mut self);
}

#[derive(Arraygen, Debug)]
#[gen_array(fn options: &mut dyn ResetOption, implicit_select_all: Option<_>)]
struct Options {
    pub a: Option<i32>,
    pub b: Option<bool>,
    pub c: f32,
}

fn casts() {
    let numbers = Numbers {
        a: -1,
        b: 1.0,
        c: -1.0,
        d: -1.0,
    };

    println!("{:?} casted to uints: {:?}", numbers, numbers.uints());
}

#[derive(Arraygen, Debug)]
#[gen_array(fn uints: u32, implicit_select_all { cast }: _)]
struct Numbers {
    pub a: i32,
    pub b: f32,
    pub c: f32,
    #[in_array(uints { override_implicit, unsafe_transmute })]
    pub d: f32,
}
