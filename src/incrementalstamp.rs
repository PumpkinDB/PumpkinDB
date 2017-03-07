//An incremental stamp that is thread safe to use 
use std::sync::Mutex;
use num_traits::One;
use num_bigint::BigUint;

//Counter will start at 1
lazy_static! {
     static ref IL_COUNT : Mutex<BigUint> = Mutex::new(One::one());
}

pub fn count() -> BigUint {
     let one : BigUint = One::one();
     let mut data =  (*IL_COUNT).lock().unwrap();
     let count = data.clone();
     *data = count.clone() + one;
     return count;
}

#[cfg(test)]
mod tests {
    use incrementalstamp;

    #[test]
    fn text_count() {
        let c1 = incrementalstamp::count();
        let c2 = incrementalstamp::count();
        assert!(c2 > c1);
    }
}