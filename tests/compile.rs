extern crate ketos;

use ketos::{Error, Interpreter, Value};
use ketos::bytecode::opcodes::*;
use ketos::name::standard_names;

fn lambda(s: &str) -> Result<Vec<u8>, Error> {
    let interp = Interpreter::new();
    let exprs = try!(interp.compile_exprs(s));
    let code = exprs.last().unwrap();

    match code.consts[1] {
        Value::Lambda(ref l) => Ok(l.code.code.clone().into_vec()),
        ref v => panic!("expected lambda; got {}", v.type_name())
    }
}

#[test]
fn test_const() {
    assert_eq!(lambda("
        (const foo (+ 1 2 3))
        (define (bar) foo)
        ").unwrap(), [
            CONST_0,
            RETURN,
        ]);
}

#[test]
fn test_const_if() {
    assert_eq!(lambda("(define (foo a) (if true (+ a) (- a)))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SYS_ARGS, standard_names::ADD.get() as u8, 1,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a) (if false (+ a) (- a)))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SYS_ARGS, standard_names::SUB.get() as u8, 1,
        RETURN,
    ]);
}

#[test]
fn test_const_fold() {
    assert_eq!(lambda("(define (foo a) (+ a 0))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SYS_ARGS, standard_names::ADD.get() as u8, 1,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a) (+ 0 a))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SYS_ARGS, standard_names::ADD.get() as u8, 1,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a) (- a 0))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SYS_ARGS, standard_names::ADD.get() as u8, 1,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a) (- 0 a))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SYS_ARGS, standard_names::SUB.get() as u8, 1,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a) (// 1 a))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SYS_ARGS, standard_names::FLOOR_DIV.get() as u8, 1,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a) (// a 1))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SYS, standard_names::FLOOR.get() as u8,
        RETURN,
    ]);
}

#[test]
fn test_dec() {
    assert_eq!(lambda("(define (foo a) (- a 1))").unwrap(), [
        LOAD_0,
        DEC,
        RETURN,
    ]);

    assert_eq!(lambda("
        (const n 1)
        (define (foo a) (- a n))
        ").unwrap(), [
            LOAD_0,
            DEC,
            RETURN,
        ]);

    assert_eq!(lambda("(define (foo a) (+ a -1))").unwrap(), [
        LOAD_0,
        DEC,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a) (+ -1 a))").unwrap(), [
        LOAD_0,
        DEC,
        RETURN,
    ]);
}

#[test]
fn test_inc() {
    assert_eq!(lambda("(define (foo a) (+ a 1))").unwrap(), [
        LOAD_0,
        INC,
        RETURN,
    ]);

    assert_eq!(lambda("
        (const n 1)
        (define (foo a) (+ a n))
        ").unwrap(), [
            LOAD_0,
            INC,
            RETURN,
        ]);

    assert_eq!(lambda("(define (foo a) (+ 1 a))").unwrap(), [
        LOAD_0,
        INC,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a) (- a -1))").unwrap(), [
        LOAD_0,
        INC,
        RETURN,
    ]);
}

#[test]
fn test_if() {
    assert_eq!(lambda("(define (foo a b c) (if a b c))").unwrap(), [
        LOAD_0,
        JUMP_IF_NOT, 5,
        LOAD_1,
        RETURN,
        LOAD_2,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo a b c) (if (not a) b c))").unwrap(), [
        LOAD_0,
        JUMP_IF, 5,
        LOAD_1,
        RETURN,
        LOAD_2,
        RETURN,
    ]);
}

#[test]
fn test_lambda() {
    assert_eq!(lambda("(define (foo a b) (+ a b))").unwrap(), [
        LOAD_PUSH_0,
        LOAD_PUSH_1,
        CALL_SYS_ARGS, standard_names::ADD.get() as u8, 2,
        RETURN,
    ]);
}

#[test]
fn test_call_self() {
    assert_eq!(lambda("(define (foo a) (do (foo a) ()))").unwrap(), [
        LOAD_PUSH_0,
        CALL_SELF, 1,
        UNIT,
        RETURN,
    ]);

    // Prevents self-call in case one wants to do something tricky.
    assert_eq!(lambda("(define (foo a) (do ((id foo) a) ()))").unwrap(), [
        GET_DEF_PUSH, 0,
        LOAD_PUSH_0,
        CALL, 1,
        UNIT,
        RETURN,
    ]);
}

#[test]
fn test_tail_recursion() {
    assert_eq!(lambda("(define (foo a) (foo a))").unwrap(), [
        LOAD_PUSH_0,
        TAIL_CALL_SELF, 1,
    ]);

    assert_eq!(lambda("(define (foo a)
                         (if (something a)
                            (foo 1)
                            (foo 2)))").unwrap(), [
        LOAD_PUSH_0,
        CALL_CONST_0, 1,
        JUMP_IF_NOT, 8,
        CONST_PUSH_1,
        TAIL_CALL_SELF, 1,
        CONST_PUSH_2,
        TAIL_CALL_SELF, 1,
    ]);

    assert_eq!(lambda("(define (foo) (and a (foo)))").unwrap(), [
        GET_DEF_0,
        JUMP_IF_NOT, 5,
        TAIL_CALL_SELF, 0,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo) (let ((a 1)) (foo)))").unwrap(), [
        CONST_PUSH_0,
        TAIL_CALL_SELF, 0,
    ]);
}

#[test]
fn test_tail_recursion_apply() {
    assert_eq!(lambda("(define (foo a) (apply foo a))").unwrap(), [
        LOAD_0,
        TAIL_APPLY_SELF, 0,
    ]);

    assert_eq!(lambda("(define (foo a)
                         (if (something a)
                            (apply foo ())
                            (apply foo ())))").unwrap(), [
        LOAD_PUSH_0,
        CALL_CONST_0, 1,
        JUMP_IF_NOT, 8,
        UNIT,
        TAIL_APPLY_SELF, 0,
        UNIT,
        TAIL_APPLY_SELF, 0,
    ]);

    assert_eq!(lambda("(define (foo) (and a (apply foo ())))").unwrap(), [
        GET_DEF_0,
        JUMP_IF_NOT, 6,
        UNIT,
        TAIL_APPLY_SELF, 0,
        RETURN,
    ]);

    assert_eq!(lambda("(define (foo) (let ((a 1)) (apply foo ())))").unwrap(), [
        CONST_PUSH_0,
        UNIT,
        TAIL_APPLY_SELF, 0,
    ]);
}
