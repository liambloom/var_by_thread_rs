use super::*;
use std::thread;
use lazy_static::lazy_static;
use std::sync::mpsc::{self, Sender, Receiver};


struct Baton;

#[test]
fn one_thread() {
    let by_thread = ByThreadCell::new();

    *by_thread.borrow_mut() = Some(2);
    assert_eq!(*by_thread.borrow(), Some(2));
}

#[test]
fn diff_between_threads() {
    lazy_static! {
        static ref BY_THREAD: ByThreadCell<i32> = ByThreadCell::new();
    }

    *BY_THREAD.borrow_mut() = Some(2);

    thread::spawn(|| {
        assert_eq!(*(&BY_THREAD).borrow(), None)
    }).join().unwrap();
}

#[test]
fn multiple_values() {
    lazy_static! {
        static ref BY_THREAD: ByThreadCell<i32> = ByThreadCell::new();
    }

    let (tx, rx): (Sender<Baton>, Receiver<Baton>) = mpsc::channel();

    assert_eq!(*BY_THREAD.borrow(), None);
    *BY_THREAD.borrow_mut() = Some(2);
    assert_eq!(*BY_THREAD.borrow(), Some(2));

    let thread = thread::spawn(move || {
        assert_eq!(*BY_THREAD.borrow(), None);
        *BY_THREAD.borrow_mut() = Some(5);
        assert_eq!(*BY_THREAD.borrow(), Some(5));
        let _ = rx.recv();
        assert_eq!(*BY_THREAD.borrow(), Some(5));
    });

    assert_eq!(*BY_THREAD.borrow(), Some(2));
    *BY_THREAD.borrow_mut() = Some(3);
    assert_eq!(*BY_THREAD.borrow(), Some(3));

    let _ = tx.send(Baton);

    let _ = thread.join();

    assert_eq!(*BY_THREAD.borrow(), Some(3));
}

#[test]
fn automatically_cleans_up() {
    lazy_static! {
        static ref BY_THREAD: ByThreadCell<i32> = ByThreadCell::new();
    }

    let mut borrow_mut = BY_THREAD.borrow_mut();
    BY_THREAD.borrow();
}

#[test]
#[should_panic]
fn bad_borrowing() {
    lazy_static! {
        static ref BY_THREAD: ByThreadCell<i32> = ByThreadCell::new();
    }

    let mut borrow_mut = BY_THREAD.borrow_mut();
    BY_THREAD.borrow();
    do_something(borrow_mut);
}

fn do_something<T>(_: T) {

}

#[test]
fn no_deadlock() {
    lazy_static! {
        static ref BY_THREAD: ByThreadCell<i32> = ByThreadCell::new();
    }

    let (tx, rx): (Sender<Baton>, Receiver<Baton>) = mpsc::channel();

    assert_eq!(*BY_THREAD.borrow(), None);
    *BY_THREAD.borrow_mut() = Some(2);
    assert_eq!(*BY_THREAD.borrow(), Some(2));

    let thread = thread::spawn(move || {
        assert_eq!(*BY_THREAD.borrow(), None);
        let mut borrow = BY_THREAD.borrow_mut();
        assert_eq!(*borrow, None);
        *borrow = Some(5);
        assert_eq!(*borrow, Some(5));
        let _ = rx.recv();
        assert_eq!(*borrow, Some(5));
        assert_eq!(*BY_THREAD.borrow(), Some(5));
    });

    assert_eq!(*BY_THREAD.borrow(), Some(2));
    let mut borrow = BY_THREAD.borrow_mut();
    assert_eq!(*borrow, Some(2));
    *borrow = Some(3);
    // assert_eq!(*BY_THREAD.borrow(), Some(3));

    let _ = tx.send(Baton);

    let _ = thread.join();
    
    assert_eq!(*borrow, Some(3));
    assert_eq!(*BY_THREAD.borrow(), Some(3));
}