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
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 3 4 5").is_ok());
    assert_eq!(vec![1, 2, 3, 4, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn can_add_two_numbers() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 +").is_ok());
    assert_eq!(vec![3], stack_to_vec(&mut f.state.stack));
}

#[test]
fn addition_error() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 +"));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("+"));
}

#[test]
fn can_subtract_two_numbers() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("3 4 -").is_ok());
    assert_eq!(vec![-1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn subtraction_error() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 -"));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("-"));
}

#[test]
fn can_multiply_two_numbers() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("2 4 *").is_ok());
    assert_eq!(vec![8], stack_to_vec(&mut f.state.stack));
}

#[test]
fn multiplication_error() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 *"));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("*"));
}

#[test]
fn can_divide_two_numbers() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("12 3 /").is_ok());
    assert_eq!(vec![4], stack_to_vec(&mut f.state.stack));
}

#[test]
fn performs_integer_division() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("8 3 /").is_ok());
    assert_eq!(vec![2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn division_error() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 /"));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("/"));
}

#[test]
fn errors_if_dividing_by_zero() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::DivisionByZero), f.evaluate_string("4 0 /"));
}

#[test]
fn addition_and_subtraction() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 + 4 -").is_ok());
    assert_eq!(vec![-1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn multiplication_and_division() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("2 4 * 3 /").is_ok());
    assert_eq!(vec![2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn dup() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 dup").is_ok());
    assert_eq!(vec![1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn dup_top_value_only() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 dup").is_ok());
    assert_eq!(vec![1, 2, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn dup_case_insensitive() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 DUP Dup dup").is_ok());
    assert_eq!(vec![1, 1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn dup_error() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("dup"));
}

#[test]
fn drop() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 drop").is_ok());
    assert_eq!(Vec::<Number>::new(), stack_to_vec(&mut f.state.stack));
}

#[test]
fn drop_with_two() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 drop").is_ok());
    assert_eq!(vec![1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn drop_case_insensitive() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 3 4 DROP Drop drop").is_ok());
    assert_eq!(vec![1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn drop_error() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("drop"));
}

#[test]
fn swap() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 swap").is_ok());
    assert_eq!(vec![2, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn swap_with_three() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 3 swap").is_ok());
    assert_eq!(vec![1, 3, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn swap_case_insensitive() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 SWAP 3 Swap 4 swap").is_ok());
    assert_eq!(vec![2, 3, 4, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn swap_error() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 swap"));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("swap"));
}

#[test]
fn over() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 over").is_ok());
    assert_eq!(vec![1, 2, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn over_with_three() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 3 over").is_ok());
    assert_eq!(vec![1, 2, 3, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn over_case_insensitive() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("1 2 OVER Over over").is_ok());
    assert_eq!(vec![1, 2, 1, 2, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn over_error() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("1 over"));
    assert_eq!(Err(Error::StackUnderflow), f.evaluate_string("over"));
}

// User-defined words

#[test]
fn can_consist_of_built_in_words() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": dup-twice dup dup ;").is_ok());
    assert!(f.evaluate_string("1 dup-twice").is_ok());
    assert_eq!(vec![1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn execute_in_the_right_order() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": countup 1 2 3 ;").is_ok());
    assert!(f.evaluate_string("countup").is_ok());
    assert_eq!(vec![1, 2, 3], stack_to_vec(&mut f.state.stack));
}

#[test]
fn redefining_an_existing_word() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo dup ;").is_ok());
    assert!(f.evaluate_string(": foo dup dup ;").is_ok());
    assert!(f.evaluate_string("1 foo").is_ok());
    assert_eq!(vec![1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn redefining_an_existing_built_in_word() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": swap dup ;").is_ok());
    assert!(f.evaluate_string("1 swap").is_ok());
    assert_eq!(vec![1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn user_defined_words_are_case_insensitive() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo dup ;").is_ok());
    assert!(f.evaluate_string("1 FOO Foo foo").is_ok());
    assert_eq!(vec![1, 1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn definitions_are_case_insensitive() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": SWAP DUP Dup dup ;").is_ok());
    assert!(f.evaluate_string("1 swap").is_ok());
    assert_eq!(vec![1, 1, 1, 1], stack_to_vec(&mut f.state.stack));
}

#[test]
fn redefining_a_built_in_operator() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": + * ;").is_ok());
    assert!(f.evaluate_string("3 4 +").is_ok());
    assert_eq!(vec![12], stack_to_vec(&mut f.state.stack));
}

#[test]
fn can_use_different_words_with_the_same_name() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo 5 ;").is_ok());
    assert!(f.evaluate_string(": bar foo ;").is_ok());
    assert!(f.evaluate_string(": foo 6 ;").is_ok());
    assert!(f.evaluate_string("bar foo").is_ok());
    assert_eq!(vec![5, 6], stack_to_vec(&mut f.state.stack));
}

// this test actually wouldn't work with recursion
#[test]
#[ignore]
fn can_define_word_that_uses_word_with_the_same_name() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo 10 ;").is_ok());
    assert!(f.evaluate_string(": foo foo 1 + ;").is_ok());
    assert!(f.evaluate_string("foo").is_ok());
    assert_eq!(vec![11], stack_to_vec(&mut f.state.stack));
}

#[test]
fn defining_a_number() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::InvalidWord), f.evaluate_string(": 1 2 ;"));
}

#[test]
fn calling_non_existing_word() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert_eq!(Err(Error::UnknownWord("FOO".to_string())), f.evaluate_string("1 foo"));
}

#[test]
fn if_statement_true() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 2 1 if dup dup else dup then 5 ;").is_ok());
    assert!(f.evaluate_string("TEST").is_ok());
    assert_eq!(vec![2, 2, 2, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn if_statement_false() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 2 0 if dup dup else dup then 5 ;").is_ok());
    assert!(f.evaluate_string("TEST").is_ok());
    assert_eq!(vec![2, 2, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn if_statement_within_definition() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": foo if dup dup else dup then 5 ;").is_ok());
    assert!(f.evaluate_string("2 1 foo").is_ok());
    assert_eq!(vec![2, 2, 2, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn nested_if_statement_true() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 2 1 1 if if dup dup then else dup then 5 ;").is_ok());
    assert!(f.evaluate_string("TEST").is_ok());
    assert_eq!(vec![2, 2, 2, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
#[ignore]
fn switch_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string("3 5 switch case 4 dup case 5 dup dup default dup dup dup then").is_ok());
    assert_eq!(vec![3, 3, 3], stack_to_vec(&mut f.state.stack));
}

#[test]
fn gcd_test() { 
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TUCK  SWAP  OVER  ; ").is_ok());
    assert!(f.evaluate_string(": GCD   ?DUP  IF  TUCK  MOD  GCD  THEN  ;").is_ok());
    assert!(f.evaluate_string("784 48 GCD").is_ok());
    assert_eq!(vec![16], stack_to_vec(&mut f.state.stack));
}

#[test]
fn value_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 5 VALUE ; ").is_ok());
    println!("defined TEST");
    assert!(f.evaluate_string("TEST FRANK FRANK").is_ok());
    println!("ran TEST");
    println!("testing");
    assert_eq!(vec![5], stack_to_vec(&mut f.state.stack));
    assert_eq!(vec![5], stack_to_vec(&mut f.state.stack));
    assert!(f.evaluate_string(": TESTING TO FRANK ; ").is_ok());
    println!("testing");
    assert_eq!(vec![5], stack_to_vec(&mut f.state.stack));
    assert!(f.evaluate_string("FRANK 10 TESTING FRANK").is_ok());
    assert_eq!(vec![5, 5, 10], stack_to_vec(&mut f.state.stack));
}

#[test]
#[ignore]
fn locals_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": reorder LOCALS| a b c | b c a ;").is_ok());
    assert!(f.evaluate_string("1 2 3 reorder").is_ok());
    assert_eq!(vec![2, 1, 3], stack_to_vec(&mut f.state.stack));
}

#[test]
fn create_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test CREATE dup dup 1 ;").is_ok());
    assert!(f.evaluate_string("2 test whatever 2 whatever ! whatever whatever @").is_ok());
    assert_eq!(vec![2, 2, 2, 1, 232, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn create_does_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test CREATE dup dup 1 DOES> dup @ ;").is_ok());
    assert!(f.evaluate_string("2 test whatever whatever DROP 5 SWAP ! whatever").is_ok());
    assert_eq!(vec![2, 2, 2, 1, 264, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn do_loop_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    println!("defining");
    assert!(f.evaluate_string(": TEST   10 0 DO  dup  LOOP ;").is_ok());
    assert_eq!(Vec::<Number>::new(), stack_to_vec(&mut f.state.stack));
    println!("testing");
    assert!(f.evaluate_string("2 TEST").is_ok());
    assert_eq!(vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn do_loop_index_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new().with_output_stream(output_stream::BufferedOutputStream::new());
    assert!(f.evaluate_string(": TEST   10 0 DO I DUP LOOP ;").is_ok());
    assert_eq!(Vec::<Number>::new(), stack_to_vec(&mut f.state.stack));
    assert!(f.evaluate_string("TEST").is_ok());
    println!("{}", f.state.output_stream.consume());
    assert_eq!(vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9], stack_to_vec(&mut f.state.stack));
}

#[test]
fn do_loop_leave_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": TEST 10 0 DO I DUP DUP 5 = IF LEAVE THEN LOOP ;").is_ok());
    assert!(f.evaluate_string("TEST").is_ok());
    assert_eq!(vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5], stack_to_vec(&mut f.state.stack));
}

#[test]
fn nested_do_loop_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new().with_output_stream(output_stream::BufferedOutputStream::new());
    assert!(f.evaluate_string(": MULTIPLICATIONS  CR 11 1 DO  DUP I * . LOOP  DROP ;").is_ok());
    assert!(f.evaluate_string(": TABLE  CR 11 1 DO  I MULTIPLICATIONS  LOOP ;").is_ok());
    assert!(f.evaluate_string("TABLE").is_ok());
    assert_eq!("\n\n1 2 3 4 5 6 7 8 9 10 \n2 4 6 8 10 12 14 16 18 20 \n3 6 9 12 15 18 21 24 27 30 \n4 8 12 16 20 24 28 32 36 40 \n5 10 15 20 25 30 35 40 45 50 \n6 12 18 24 30 36 42 48 54 60 \n7 14 21 28 35 42 49 56 63 70 \n8 16 24 32 40 48 56 64 72 80 \n9 18 27 36 45 54 63 72 81 90 \n10 20 30 40 50 60 70 80 90 100 ", f.state.output_stream.consume());
}

#[test]
fn pentajumps_loop_plus() {
    let mut f = Forth::<kernels::DefaultKernel>::new().with_output_stream(output_stream::BufferedOutputStream::new());
    assert!(f.evaluate_string(": PENTAJUMPS  50 0 DO  I .  5 +LOOP ;").is_ok());
    assert!(f.evaluate_string("PENTAJUMPS").is_ok());
    assert_eq!("0 5 10 15 20 25 30 35 40 45 ", f.state.output_stream.consume());
}

#[test]
fn begin_until_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test 0 BEGIN 1 + DUP DUP 5 = UNTIL ;").is_ok());
    assert!(f.evaluate_string("2 test").is_ok());
    assert_eq!(vec![2, 1, 2, 3, 4, 5, 5], stack_to_vec(&mut f.state.stack));    
}

#[test]
fn while_repeat_loop_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test 0 BEGIN 1 + DUP 5 < WHILE DUP 2 * SWAP REPEAT ;").is_ok());
    assert!(f.evaluate_string("2 test").is_ok());
    assert_eq!(vec![2, 2, 4, 6, 8, 5], stack_to_vec(&mut f.state.stack));    
}

#[test]
fn print_string_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new().with_output_stream(output_stream::BufferedOutputStream::new());
    assert!(f.evaluate_string(": test .\" hello world this is a test \" 2 2 + . ;").is_ok());
    assert!(f.evaluate_string("test").is_ok());
    assert_eq!(f.state.output_stream.consume(), "hello world this is a test 4 ");
}

#[test]
fn literal_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": FOUR-MORE  [ 4 ] LITERAL + ;").is_ok());
    assert!(f.evaluate_string("2 FOUR-MORE").is_ok());
    assert_eq!(vec![6], stack_to_vec(&mut f.state.stack));    
}

#[test]
fn test_brackets() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": test [ 3 4 + ] dup ;").is_ok());
    assert!(f.evaluate_string("2 test").is_ok());
    assert_eq!(vec![7, 2, 2], stack_to_vec(&mut f.state.stack));
}

#[test]
fn print_size_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new().with_output_stream(output_stream::BufferedOutputStream::new());
    assert!(f.evaluate_string("-1 -1 UM+ D.").is_ok());
    assert_eq!("36893488147419103230 ", f.state.output_stream.consume());    
}

#[test]
fn custom_constant_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new();
    assert!(f.evaluate_string(": const create , does> @ ;").is_ok());
    assert!(f.evaluate_string("5 const frank frank").is_ok());
    assert_eq!(vec![5], stack_to_vec(&mut f.state.stack));    
}

#[test]
fn materials_program_test() {
    let mut f = Forth::<kernels::DefaultKernel>::new().with_output_stream(output_stream::BufferedOutputStream::new());
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
       \" clay\"             120        727  MATERIAL CLAY").is_ok());
    assert!(f.evaluate_string("cement 10 foot pile 10 foot 3 inch pile dry-sand 10 foot pile").is_ok());
    assert_eq!("= 138 tons of cement= 151 tons of cement= 81 tons of dry sand", f.state.output_stream.consume());
}
