use rand::{thread_rng, Rng};

pub fn greet() {
    print!("Hi!")
}

pub fn random_number() -> i32 {
    thread_rng().gen_range(0, 100)
}
