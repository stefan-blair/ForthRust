use forth::{Error, Forth, Number, output_stream, stack, kernels};


pub fn stack_to_vec(stack: &stack::Stack) -> Vec<Number> {
    stack.to_vec().iter().map(|x| x.to_number()).collect::<Vec<_>>()
}


#[test]
fn no_input_no_stack() {
    assert_eq!(Vec::<Number>::new(), stack_to_vec(&mut Forth::<kernels::DefaultKernel>::new().state.stack));
}

#[test]
fn numbers_just_get_pushed_onto_the_stack() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 3 4 5", &mut output_stream).is_ok());
    assert_eq!(vec![1, 2, 3, 4, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn can_add_two_numbers() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 +", &mut output_stream).is_ok());
    assert_eq!(vec![3], stack_to_vec(&mut f.state.stack));
}

#[test]
fn addition_error() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 +", &mut output_stream));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("+", &mut output_stream));
}

#[test]
fn can_subtract_two_numbers() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("3 4 -", &mut output_stream).is_ok());
    assert_eq!(vec![-1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn subtraction_error() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 -", &mut output_stream));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("-", &mut output_stream));
}

#[test]
fn can_multiply_two_numbers() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("2 4 *", &mut output_stream).is_ok());
    assert_eq!(vec![8], stack_to_vec(&mut f.state.stack));
}

#[test]
fn multiplication_error() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 *", &mut output_stream));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("*", &mut output_stream));
}

#[test]
fn can_divide_two_numbers() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("12 3 /", &mut output_stream).is_ok());
    assert_eq!(vec![4], stack_to_vec(&mut f.state.stack));
}

#[test]
fn performs_integer_division() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("8 3 /", &mut output_stream).is_ok());
    assert_eq!(vec![2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn division_error() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 /", &mut output_stream));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("/", &mut output_stream));
}

#[test]
fn errors_if_dividing_by_zero() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::DivisionByZero), f.evaluate_string("4 0 /", &mut output_stream));
}

#[test]
fn addition_and_subtraction() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 + 4 -", &mut output_stream).is_ok());
    assert_eq!(vec![-1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn multiplication_and_division() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("2 4 * 3 /", &mut output_stream).is_ok());
    assert_eq!(vec![2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn dup() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 dup", &mut output_stream).is_ok());
    assert_eq!(vec![1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn dup_top_value_only() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 dup", &mut output_stream).is_ok());
    assert_eq!(vec![1, 2, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn dup_case_insensitive() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 DUP Dup dup", &mut output_stream).is_ok());
    assert_eq!(vec![1, 1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn dup_error() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("dup", &mut output_stream));
}

#[test]
fn drop() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 drop", &mut output_stream).is_ok());
    assert_eq!(Vec::<Number>::new(), stack_to_vec(&mut f.state.stack));
}

#[test]
fn drop_with_two() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 drop", &mut output_stream).is_ok());
    assert_eq!(vec![1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn drop_case_insensitive() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 3 4 DROP Drop drop", &mut output_stream).is_ok());
    assert_eq!(vec![1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn drop_error() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("drop", &mut output_stream));
}

#[test]
fn swap() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 swap", &mut output_stream).is_ok());
    assert_eq!(vec![2, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn swap_with_three() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 3 swap", &mut output_stream).is_ok());
    assert_eq!(vec![1, 3, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn swap_case_insensitive() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 SWAP 3 Swap 4 swap", &mut output_stream).is_ok());
    assert_eq!(vec![2, 3, 4, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn swap_error() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 swap", &mut output_stream));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("swap", &mut output_stream));
}

#[test]
fn over() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 over", &mut output_stream).is_ok());
    assert_eq!(vec![1, 2, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn over_with_three() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 3 over", &mut output_stream).is_ok());
    assert_eq!(vec![1, 2, 3, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn over_case_insensitive() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 OVER Over over", &mut output_stream).is_ok());
    assert_eq!(vec![1, 2, 1, 2, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn over_error() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 over", &mut output_stream));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("over", &mut output_stream));
}

// User-defined words

#[test]
fn can_consist_of_built_in_words() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": dup-twice dup dup ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("1 dup-twice", &mut output_stream).is_ok());
    assert_eq!(vec![1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn execute_in_the_right_order() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": countup 1 2 3 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("countup", &mut output_stream).is_ok());
    assert_eq!(vec![1, 2, 3], stack_to_vec(&mut f.state.stack));
}

#[test]
fn redefining_an_existing_word() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo dup ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string(": foo dup dup ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("1 foo", &mut output_stream).is_ok());
    assert_eq!(vec![1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn redefining_an_existing_built_in_word() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": swap dup ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("1 swap", &mut output_stream).is_ok());
    assert_eq!(vec![1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn user_defined_words_are_case_insensitive() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo dup ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("1 FOO Foo foo", &mut output_stream).is_ok());
    assert_eq!(vec![1, 1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn definitions_are_case_insensitive() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": SWAP DUP Dup dup ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("1 swap", &mut output_stream).is_ok());
    assert_eq!(vec![1, 1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn redefining_a_built_in_operator() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": + * ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("3 4 +", &mut output_stream).is_ok());
    assert_eq!(vec![12], stack_to_vec(&mut f.state.stack));
}

#[test]
fn can_use_different_words_with_the_same_name() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo 5 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string(": bar foo ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string(": foo 6 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("bar foo", &mut output_stream).is_ok());
    assert_eq!(vec![5, 6], stack_to_vec(&mut f.state.stack));
}

// this test actually wouldn't work with recursion
#[test]
#[ignore]
fn can_define_word_that_uses_word_with_the_same_name() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo 10 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string(": foo foo 1 + ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("foo", &mut output_stream).is_ok());
    assert_eq!(vec![11], stack_to_vec(&mut f.state.stack));
}

#[test]
fn defining_a_number() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::InvalidWord), f.evaluate_string(": 1 2 ;", &mut output_stream));
}

#[test]
fn calling_non_existing_word() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::UnknownWord("FOO".to_string())), f.evaluate_string("1 foo", &mut output_stream));
}

#[test]
fn if_statement_true() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 2 1 if dup dup else dup then 5 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("TEST", &mut output_stream).is_ok());
    assert_eq!(vec![2, 2, 2, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn if_statement_false() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 2 0 if dup dup else dup then 5 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("TEST", &mut output_stream).is_ok());
    assert_eq!(vec![2, 2, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn if_statement_within_definition() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo if dup dup else dup then 5 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("2 1 foo", &mut output_stream).is_ok());
    assert_eq!(vec![2, 2, 2, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn nested_if_statement_true() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 2 1 1 if if dup dup then else dup then 5 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("TEST", &mut output_stream).is_ok());
    assert_eq!(vec![2, 2, 2, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
#[ignore]
fn switch_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("3 5 switch case 4 dup case 5 dup dup default dup dup dup then", &mut output_stream).is_ok());
    assert_eq!(vec![3, 3, 3], stack_to_vec(&mut f.state.stack));
}

#[test]
fn gcd_test() { 
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TUCK  SWAP  OVER  ; ", &mut output_stream).is_ok());
    assert!(f.evaluate_string(": GCD   ?DUP  IF  TUCK  MOD  GCD  THEN  ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("784 48 GCD", &mut output_stream).is_ok());
    assert_eq!(vec![16], stack_to_vec(&mut f.state.stack));
}

#[test]
fn value_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 5 VALUE ; ", &mut output_stream).is_ok());
    println!("defined TEST");
    assert!(f.evaluate_string("TEST FRANK FRANK", &mut output_stream).is_ok());
    println!("ran TEST");
    println!("testing");
    assert_eq!(vec![5], stack_to_vec(&mut f.state.stack));
    assert_eq!(vec![5], stack_to_vec(&mut f.state.stack));
    assert!(f.evaluate_string(": TESTING TO FRANK ; ", &mut output_stream).is_ok());
    println!("testing");
    assert_eq!(vec![5], stack_to_vec(&mut f.state.stack));
    assert!(f.evaluate_string("FRANK 10 TESTING FRANK", &mut output_stream).is_ok());
    assert_eq!(vec![5, 5, 10], stack_to_vec(&mut f.state.stack));
}

#[test]
#[ignore]
fn locals_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": reorder LOCALS| a b c | b c a ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("1 2 3 reorder", &mut output_stream).is_ok());
    assert_eq!(vec![2, 1, 3], stack_to_vec(&mut f.state.stack));
}

#[test]
fn create_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test CREATE dup dup 1 ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("2 test whatever 2 whatever ! whatever whatever @", &mut output_stream).is_ok());
    assert_eq!(vec![2, 2, 2, 1, 232, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn create_does_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test CREATE dup dup 1 DOES> dup @ ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("2 test whatever whatever DROP 5 SWAP ! whatever", &mut output_stream).is_ok());
    assert_eq!(vec![2, 2, 2, 1, 264, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn do_loop_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    println!("defining");
    assert!(f.evaluate_string(": TEST   10 0 DO  dup  LOOP ;", &mut output_stream).is_ok());
    assert_eq!(Vec::<Number>::new(), stack_to_vec(&mut f.state.stack));
    println!("testing");
    assert!(f.evaluate_string("2 TEST", &mut output_stream).is_ok());
    assert_eq!(vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn do_loop_index_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST   10 0 DO I DUP LOOP ;", &mut output_stream).is_ok());
    assert_eq!(Vec::<Number>::new(), stack_to_vec(&mut f.state.stack));
    assert!(f.evaluate_string("TEST", &mut output_stream).is_ok());
    println!("{}", output_stream.consume());
    assert_eq!(vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9], stack_to_vec(&mut f.state.stack));
}

#[test]
fn do_loop_leave_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 10 0 DO I DUP DUP 5 = IF LEAVE THEN LOOP ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("TEST", &mut output_stream).is_ok());
    assert_eq!(vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn nested_do_loop_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": MULTIPLICATIONS  CR 11 1 DO  DUP I * . LOOP  DROP ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string(": TABLE  CR 11 1 DO  I MULTIPLICATIONS  LOOP ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("TABLE", &mut output_stream).is_ok());
    assert_eq!("\n\n1 2 3 4 5 6 7 8 9 10 \n2 4 6 8 10 12 14 16 18 20 \n3 6 9 12 15 18 21 24 27 30 \n4 8 12 16 20 24 28 32 36 40 \n5 10 15 20 25 30 35 40 45 50 \n6 12 18 24 30 36 42 48 54 60 \n7 14 21 28 35 42 49 56 63 70 \n8 16 24 32 40 48 56 64 72 80 \n9 18 27 36 45 54 63 72 81 90 \n10 20 30 40 50 60 70 80 90 100 ", output_stream.consume());
}

#[test]
fn pentajumps_loop_plus() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": PENTAJUMPS  50 0 DO  I .  5 +LOOP ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("PENTAJUMPS", &mut output_stream).is_ok());
    assert_eq!("0 5 10 15 20 25 30 35 40 45 ", output_stream.consume());
}

#[test]
fn begin_until_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test 0 BEGIN 1 + DUP DUP 5 = UNTIL ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("2 test", &mut output_stream).is_ok());
    assert_eq!(vec![2, 1, 2, 3, 4, 5, 5], stack_to_vec(&mut f.state.stack));    
}

#[test]
fn while_repeat_loop_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test 0 BEGIN 1 + DUP 5 < WHILE DUP 2 * SWAP REPEAT ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("2 test", &mut output_stream).is_ok());
    assert_eq!(vec![2, 2, 4, 6, 8, 5], stack_to_vec(&mut f.state.stack));    
}

#[test]
fn print_string_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test .\" hello world this is a test \" 2 2 + . ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("test", &mut output_stream).is_ok());
    assert_eq!(output_stream.consume(), "hello world this is a test 4 ");
}

#[test]
fn literal_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": FOUR-MORE  [ 4 ] LITERAL + ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("2 FOUR-MORE", &mut output_stream).is_ok());
    assert_eq!(vec![6], stack_to_vec(&mut f.state.stack));    
}

#[test]
fn test_brackets() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test [ 3 4 + ] dup ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("2 test", &mut output_stream).is_ok());
    assert_eq!(vec![7, 2, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn print_size_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("-1 -1 UM+ D.", &mut output_stream).is_ok());
    assert_eq!("36893488147419103230 ", output_stream.consume());    
}

#[test]
fn custom_constant_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": const create , does> @ ;", &mut output_stream).is_ok());
    assert!(f.evaluate_string("5 const frank frank", &mut output_stream).is_ok());
    assert_eq!(vec![5], stack_to_vec(&mut f.state.stack));    
}

#[test]
fn materials_program_test() {
    let mut output_stream = output_stream::BufferedOutputStream::new();
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("\\ \"No Weighting\" from Starting Forth Chapter 12
    VARIABLE DENSITY
    VARIABLE THETA
    VARIABLE ID
    
    : \" ( -- addr )   [CHAR] \" WORD DUP C@ 1+ ALLOT ;
    
    : MATERIAL ( addr n1 n2 -- )    \\ addr=string, n1=density, n2=theta
       CREATE  , , , 
       DOES> ( -- )   DUP @ THETA !
       CELL+ DUP @ DENSITY !  CELL+ @ ID ! ;
    
    : .SUBSTANCE ( -- )   ID @ COUNT TYPE ;
    : FOOT ( n1 -- n2 )   10 * ;
    : INCH ( n1 -- n2 )   100 12 */  5 +  10 /  + ;
    : /TAN ( n1 -- n2 )   1000 THETA @ */ ;
    
    : PILE ( n -- )         \\ n=scaled height
       DUP DUP 10 */ 1000 */  355 339 */  /TAN /TAN
       DENSITY @ 200 */  .\" = \" . .\" tons of \"  .SUBSTANCE ;
    
    \\ table of materials
    \\   string-address  density  tan[theta] 
       \" cement\"           131        700  MATERIAL CEMENT
       \" loose gravel\"      93        649  MATERIAL LOOSE-GRAVEL
       \" packed gravel\"    100        700  MATERIAL PACKED-GRAVEL
       \" dry sand\"          90        754  MATERIAL DRY-SAND
       \" wet sand\"         118        900  MATERIAL WET-SAND
       \" clay\"             120        727  MATERIAL CLAY", &mut output_stream).is_ok());
    assert!(f.evaluate_string("cement 10 foot pile 10 foot 3 inch pile dry-sand 10 foot pile", &mut output_stream).is_ok());
    assert_eq!("= 138 tons of cement= 151 tons of cement= 81 tons of dry sand", output_stream.consume());
}
