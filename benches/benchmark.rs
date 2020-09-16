#![feature(test)]
extern crate forth;
extern crate test;

use forth::{Forth, kernels};
use test::Bencher;

// literal: sequential accesses and reuse accesses
#[bench]
fn embedded_literal_sequential_test(b: &mut Bencher) {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    b.iter(move || {
        // create literals
        for i in 0..800 {
            assert!(f.evaluate_string(&format!("{} : test{} literal . ;", i, i)).is_ok());
        }

        for _ in 0..100 {
            for i in 0..800 {
                assert!(f.evaluate_string(&format!("test{}", i)).is_ok());
            }    
        }
    });
}

#[bench]
fn embedded_literal_reuse_test(b: &mut Bencher) {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    b.iter(move || {
        // create literals
        for i in 0..800 {
            assert!(f.evaluate_string(&format!("{} : test{} literal . ;", i, i)).is_ok());
        }

        for i in 0..800 {
            for _ in 0..100 {
                assert!(f.evaluate_string(&format!("test{}", i)).is_ok());
            }    
        }
    });
}

#[bench]
fn compiled_literal_sequential_test(b: &mut Bencher) {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    b.iter(move || {
        // create literals
        for i in 0..800 {
            assert!(f.evaluate_string(&format!("{} : test{} _literal . ;", i, i)).is_ok());
        }

        for _ in 0..100 {
            for i in 0..800 {
                assert!(f.evaluate_string(&format!("test{}", i)).is_ok());
            }    
        }
    });
}

#[bench]
fn compiled_literal_reuse_test(b: &mut Bencher) {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    b.iter(move || {
        // create literals
        for i in 0..800 {
            assert!(f.evaluate_string(&format!("{} : test{} _literal . ;", i, i)).is_ok());
        }

        for i in 0..800 {
            for _ in 0..100 {
                assert!(f.evaluate_string(&format!("test{}", i)).is_ok());
            }    
        }
    });
}
