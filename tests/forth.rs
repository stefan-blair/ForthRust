use forth::{Error, Forth, Number};


#[test]
fn no_input_no_stack() {
    assert_eq!(Vec::<Number>::new(), Forth::new().stack());
}

#[test]
fn numbers_just_get_pushed_onto_the_stack() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 3 4 5").is_ok());
    assert_eq!(vec![1, 2, 3, 4, 5], f.stack());
}

#[test]
fn can_add_two_numbers() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 +").is_ok());
    assert_eq!(vec![3], f.stack());
}

#[test]
fn addition_error() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::StackUnderflow), f.eval("1 +"));
    assert_eq!(Err(Error::StackUnderflow), f.eval("+"));
}

#[test]
fn can_subtract_two_numbers() {
    let mut f = Forth::new();
    assert!(f.eval("3 4 -").is_ok());
    assert_eq!(vec![-1], f.stack());
}

#[test]
fn subtraction_error() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::StackUnderflow), f.eval("1 -"));
    assert_eq!(Err(Error::StackUnderflow), f.eval("-"));
}

#[test]
fn can_multiply_two_numbers() {
    let mut f = Forth::new();
    assert!(f.eval("2 4 *").is_ok());
    assert_eq!(vec![8], f.stack());
}

#[test]
fn multiplication_error() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::StackUnderflow), f.eval("1 *"));
    assert_eq!(Err(Error::StackUnderflow), f.eval("*"));
}

#[test]
fn can_divide_two_numbers() {
    let mut f = Forth::new();
    assert!(f.eval("12 3 /").is_ok());
    assert_eq!(vec![4], f.stack());
}

#[test]
fn performs_integer_division() {
    let mut f = Forth::new();
    assert!(f.eval("8 3 /").is_ok());
    assert_eq!(vec![2], f.stack());
}

#[test]
fn division_error() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::StackUnderflow), f.eval("1 /"));
    assert_eq!(Err(Error::StackUnderflow), f.eval("/"));
}

#[test]
fn errors_if_dividing_by_zero() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::DivisionByZero), f.eval("4 0 /"));
}

#[test]
fn addition_and_subtraction() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 + 4 -").is_ok());
    assert_eq!(vec![-1], f.stack());
}

#[test]
fn multiplication_and_division() {
    let mut f = Forth::new();
    assert!(f.eval("2 4 * 3 /").is_ok());
    assert_eq!(vec![2], f.stack());
}

#[test]
fn dup() {
    let mut f = Forth::new();
    assert!(f.eval("1 dup").is_ok());
    assert_eq!(vec![1, 1], f.stack());
}

#[test]
fn dup_top_value_only() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 dup").is_ok());
    assert_eq!(vec![1, 2, 2], f.stack());
}

#[test]
fn dup_case_insensitive() {
    let mut f = Forth::new();
    assert!(f.eval("1 DUP Dup dup").is_ok());
    assert_eq!(vec![1, 1, 1, 1], f.stack());
}

#[test]
fn dup_error() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::StackUnderflow), f.eval("dup"));
}

#[test]
fn drop() {
    let mut f = Forth::new();
    assert!(f.eval("1 drop").is_ok());
    assert_eq!(Vec::<Number>::new(), f.stack());
}

#[test]
fn drop_with_two() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 drop").is_ok());
    assert_eq!(vec![1], f.stack());
}

#[test]
fn drop_case_insensitive() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 3 4 DROP Drop drop").is_ok());
    assert_eq!(vec![1], f.stack());
}

#[test]
fn drop_error() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::StackUnderflow), f.eval("drop"));
}

#[test]
fn swap() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 swap").is_ok());
    assert_eq!(vec![2, 1], f.stack());
}

#[test]
fn swap_with_three() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 3 swap").is_ok());
    assert_eq!(vec![1, 3, 2], f.stack());
}

#[test]
fn swap_case_insensitive() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 SWAP 3 Swap 4 swap").is_ok());
    assert_eq!(vec![2, 3, 4, 1], f.stack());
}

#[test]
fn swap_error() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::StackUnderflow), f.eval("1 swap"));
    assert_eq!(Err(Error::StackUnderflow), f.eval("swap"));
}

#[test]
fn over() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 over").is_ok());
    assert_eq!(vec![1, 2, 1], f.stack());
}

#[test]
fn over_with_three() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 3 over").is_ok());
    assert_eq!(vec![1, 2, 3, 2], f.stack());
}

#[test]
fn over_case_insensitive() {
    let mut f = Forth::new();
    assert!(f.eval("1 2 OVER Over over").is_ok());
    assert_eq!(vec![1, 2, 1, 2, 1], f.stack());
}

#[test]
fn over_error() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::StackUnderflow), f.eval("1 over"));
    assert_eq!(Err(Error::StackUnderflow), f.eval("over"));
}

// User-defined words

#[test]
fn can_consist_of_built_in_words() {
    let mut f = Forth::new();
    assert!(f.eval(": dup-twice dup dup ;").is_ok());
    assert!(f.eval("1 dup-twice").is_ok());
    assert_eq!(vec![1, 1, 1], f.stack());
}

#[test]
fn execute_in_the_right_order() {
    let mut f = Forth::new();
    assert!(f.eval(": countup 1 2 3 ;").is_ok());
    assert!(f.eval("countup").is_ok());
    assert_eq!(vec![1, 2, 3], f.stack());
}

#[test]
fn redefining_an_existing_word() {
    let mut f = Forth::new();
    assert!(f.eval(": foo dup ;").is_ok());
    assert!(f.eval(": foo dup dup ;").is_ok());
    assert!(f.eval("1 foo").is_ok());
    assert_eq!(vec![1, 1, 1], f.stack());
}

#[test]
fn redefining_an_existing_built_in_word() {
    let mut f = Forth::new();
    assert!(f.eval(": swap dup ;").is_ok());
    assert!(f.eval("1 swap").is_ok());
    assert_eq!(vec![1, 1], f.stack());
}

#[test]
fn user_defined_words_are_case_insensitive() {
    let mut f = Forth::new();
    assert!(f.eval(": foo dup ;").is_ok());
    assert!(f.eval("1 FOO Foo foo").is_ok());
    assert_eq!(vec![1, 1, 1, 1], f.stack());
}

#[test]
fn definitions_are_case_insensitive() {
    let mut f = Forth::new();
    assert!(f.eval(": SWAP DUP Dup dup ;").is_ok());
    assert!(f.eval("1 swap").is_ok());
    assert_eq!(vec![1, 1, 1, 1], f.stack());
}

#[test]
fn redefining_a_built_in_operator() {
    let mut f = Forth::new();
    assert!(f.eval(": + * ;").is_ok());
    assert!(f.eval("3 4 +").is_ok());
    assert_eq!(vec![12], f.stack());
}

#[test]
fn can_use_different_words_with_the_same_name() {
    let mut f = Forth::new();
    assert!(f.eval(": foo 5 ;").is_ok());
    assert!(f.eval(": bar foo ;").is_ok());
    assert!(f.eval(": foo 6 ;").is_ok());
    assert!(f.eval("bar foo").is_ok());
    assert_eq!(vec![5, 6], f.stack());
}

// this test actually wouldn't work with recursion
#[test]
#[ignore]
fn can_define_word_that_uses_word_with_the_same_name() {
    let mut f = Forth::new();
    assert!(f.eval(": foo 10 ;").is_ok());
    assert!(f.eval(": foo foo 1 + ;").is_ok());
    assert!(f.eval("foo").is_ok());
    assert_eq!(vec![11], f.stack());
}

#[test]
fn defining_a_number() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::InvalidWord), f.eval(": 1 2 ;"));
}

#[test]
fn calling_non_existing_word() {
    let mut f = Forth::new();
    assert_eq!(Err(Error::UnknownWord), f.eval("1 foo"));
}

#[test]
fn if_statement_true() {
    let mut f = Forth::new();
    assert!(f.eval(": TEST 2 1 if dup dup else dup then 5 ;").is_ok());
    assert!(f.eval("TEST").is_ok());
    assert_eq!(vec![2, 2, 2, 5], f.stack());
}

#[test]
fn if_statement_false() {
    let mut f = Forth::new();
    assert!(f.eval(": TEST 2 0 if dup dup else dup then 5 ;").is_ok());
    assert!(f.eval("TEST").is_ok());
    assert_eq!(vec![2, 2, 5], f.stack());
}

#[test]
fn if_statement_within_definition() {
    let mut f = Forth::new();
    assert!(f.eval(": foo if dup dup else dup then 5 ;").is_ok());
    assert!(f.eval("2 1 foo").is_ok());
    assert_eq!(vec![2, 2, 2, 5], f.stack());
}

#[test]
fn nested_if_statement_true() {
    let mut f = Forth::new();
    assert!(f.eval(": TEST 2 1 1 if if dup dup then else dup then 5 ;").is_ok());
    assert!(f.eval("TEST").is_ok());
    assert_eq!(vec![2, 2, 2, 5], f.stack());
}

#[test]
#[ignore]
fn switch_test() {
    let mut f = Forth::new();
    assert!(f.eval("3 5 switch case 4 dup case 5 dup dup default dup dup dup then").is_ok());
    assert_eq!(vec![3, 3, 3], f.stack());
}

#[test]
fn gcd_test() { 
    let mut f = Forth::new();
    assert!(f.eval(": TUCK  SWAP  OVER  ; ").is_ok());
    assert!(f.eval(": GCD   ?DUP  IF  TUCK  MOD  GCD  THEN  ;").is_ok());
    assert!(f.eval("784 48 GCD").is_ok());
    assert_eq!(vec![16], f.stack());
}

#[test]
fn value_test() {
    let mut f = Forth::new();
    assert!(f.eval(": TEST 5 VALUE ; ").is_ok());
    assert!(f.eval("TEST FRANK FRANK").is_ok());
    assert!(f.eval(": TESTING TO FRANK ; ").is_ok());
    assert!(f.eval("FRANK 10 TESTING FRANK").is_ok());
    assert_eq!(vec![5, 5, 10], f.stack());
}

#[test]
#[ignore]
fn locals_test() {
    let mut f = Forth::new();
    assert!(f.eval(": reorder LOCALS| a b c | b c a ;").is_ok());
    assert!(f.eval("1 2 3 reorder").is_ok());
    assert_eq!(vec![2, 1, 3], f.stack());
}

#[test]
fn create_test() {
    let mut f = Forth::new();
    assert!(f.eval(": test CREATE dup dup 1 ;").is_ok());
    assert!(f.eval("2 test whatever 2 whatever ! whatever whatever @").is_ok());
    assert_eq!(vec![2, 2, 2, 1, 232, 2], f.stack_numbers());
}

#[test]
fn create_does_test() {
    let mut f = Forth::new();
    assert!(f.eval(": test CREATE dup dup 1 DOES> dup @ ;").is_ok());
    assert!(f.eval("2 test whatever whatever DROP 5 SWAP ! whatever").is_ok());
    assert_eq!(vec![2, 2, 2, 1, 264, 5], f.stack_numbers());
}

#[test]
fn do_loop_test() {
    let mut f = Forth::new();
    println!("defining");
    assert!(f.eval(": TEST   10 0 DO  dup  LOOP ;").is_ok());
    assert_eq!(Vec::<Number>::new(), f.stack());
    println!("testing");
    assert!(f.eval("2 TEST").is_ok());
    assert_eq!(vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], f.stack());
}

#[test]
fn do_loop_index_test() {
    let mut f = Forth::new();
    assert!(f.eval(": TEST   10 0 DO I DUP LOOP ;").is_ok());
    assert_eq!(Vec::<Number>::new(), f.stack());
    assert!(f.eval("TEST").is_ok());
    assert_eq!(vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9], f.stack());
}

#[test]
fn do_loop_leave_test() {
    let mut f = Forth::new();
    assert!(f.eval(": TEST 10 0 DO I DUP DUP 5 = IF LEAVE THEN LOOP ;").is_ok());
    assert!(f.eval("TEST").is_ok());
    assert_eq!(vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5], f.stack());
}

#[test]
fn nested_do_loop_test() {
    let mut f = Forth::new();
    assert!(f.eval(": MULTIPLICATIONS  CR 11 1 DO  DUP I * . LOOP  DROP ;").is_ok());
    assert!(f.eval(": TABLE  CR 11 1 DO  I MULTIPLICATIONS  LOOP ;").is_ok());
    assert!(f.eval("TABLE").is_ok());
    assert_eq!("\n\n1 2 3 4 5 6 7 8 9 10 \n2 4 6 8 10 12 14 16 18 20 \n3 6 9 12 15 18 21 24 27 30 \n4 8 12 16 20 24 28 32 36 40 \n5 10 15 20 25 30 35 40 45 50 \n6 12 18 24 30 36 42 48 54 60 \n7 14 21 28 35 42 49 56 63 70 \n8 16 24 32 40 48 56 64 72 80 \n9 18 27 36 45 54 63 72 81 90 \n10 20 30 40 50 60 70 80 90 100 ", f.consume_output());
}

#[test]
fn pentajumps_loop_plus() {
    let mut f = Forth::new();
    assert!(f.eval(": PENTAJUMPS  50 0 DO  I .  5 +LOOP ;").is_ok());
    assert!(f.eval("PENTAJUMPS").is_ok());
    assert_eq!("0 5 10 15 20 25 30 35 40 45 ", f.consume_output());
}

#[test]
fn begin_until_test() {
    let mut f = Forth::new();
    assert!(f.eval(": test 0 BEGIN 1 + DUP DUP 5 = UNTIL ;").is_ok());
    assert!(f.eval("2 test").is_ok());
    assert_eq!(vec![2, 1, 2, 3, 4, 5, 5], f.stack());    
}

#[test]
fn while_repeat_loop_test() {
    let mut f = Forth::new();
    assert!(f.eval(": test 0 BEGIN 1 + DUP 5 < WHILE DUP 2 * SWAP REPEAT ;").is_ok());
    assert!(f.eval("2 test").is_ok());
    assert_eq!(vec![2, 2, 4, 6, 8, 5], f.stack());    
}

#[test]
fn literal_test() {
    let mut f = Forth::new();
    assert!(f.eval(": FOUR-MORE  [ 4 ] LITERAL + ;").is_ok());
    assert!(f.eval("2 FOUR-MORE").is_ok());
    assert_eq!(vec![6], f.stack());    
}

#[test]
fn test_brackets() {
    let mut f = Forth::new();
    assert!(f.eval(": test [ 3 4 + ] dup ;").is_ok());
    assert!(f.eval("2 test").is_ok());
    assert_eq!(vec![7, 2, 2], f.stack());
}

#[test]
fn print_size_test() {
    let mut f = Forth::new();
    assert!(f.eval("-1 -1 UM+ D.").is_ok());
    assert_eq!("36893488147419103230 ", f.consume_output());    
}