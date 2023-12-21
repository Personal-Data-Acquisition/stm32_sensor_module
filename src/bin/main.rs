#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(type_alias_impl_trait)]
#![test_runner(my_runner)]

use {defmt_rtt as _, panic_probe as _};

fn my_runner(tests: &[&i32]) {
    for t in tests {
        if **t == 0 {
            //defmt::println!("PASSED");
        } else {
            //defmt::println!("FAILED");
        }
    }
}




#[test_case]
const WILL_PASS: i32 = 0;

#[test_case]
const WILL_FAIL: i32 = 4;


