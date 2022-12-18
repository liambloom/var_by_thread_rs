use std::{collections::HashMap, thread::{ThreadId, self}, cell::{UnsafeCell, Cell, RefCell, self}, ops::{Deref, DerefMut}, marker::PhantomData, rc::Rc};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};


// TODO: Implement my own HashMap that makes no assumptions about what is being mutated when
// and can be used without the RwLock

#[cfg(test)]
mod tests;

type TheMap<T> = HashMap<ThreadId, UnsafeCell<(Box<RefCell<()>>, Option<T>)>>;

pub struct ByThreadCell<T> {
    value: RwLock<TheMap<T>>,
}

unsafe impl<T> Sync for ByThreadCell<T> { }

impl<T> ByThreadCell<T> {
    pub fn new() -> Self {
        Self { value: RwLock::new(TheMap::new()) }
    }

    pub fn borrow<'a>(&self) -> Ref<'_, T> {
        // let bar: RwLockReadGuard<'a, ()> = self.value.read().get(&thread::current().id()).unwrap().0.read();
        Ref::new(self.value.read())
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.value.write().entry(thread::current().id()).or_insert(UnsafeCell::new((Box::new(RefCell::new(())), None)));

        RefMut::new(self.value.read())
    }
}

pub struct Ref<'a, T> {
    value_lock: Option<*mut cell::Ref<'a, ()>>,
    map_lock: RwLockReadGuard<'a, TheMap<T>>,

    // The following note and property are modified from 
    // https://github.com/kinghajj/deque/blob/master/src/lib.rs#L67
    //
    // Marker so that the Ref is not Sync. The Ref can only be
    // accessed from a single thread at once. Ideally we would use a negative
    // impl here but these are not stable yet.
    phantom: PhantomData<Cell<()>>,
}

impl<'a, T> Ref<'a, T> {
    fn new(lock: RwLockReadGuard<'a, TheMap<T>>) -> Self {
        Ref { 
            value_lock: lock.get(&thread::current().id()).map(|t_l| Box::into_raw(Box::new(unsafe { (*t_l.get()).0.borrow() }))), 
            map_lock: lock, 
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = Option<T>;

    fn deref(&self) -> &'a Self::Target {
        match self.map_lock.get(&thread::current().id()) {
            Some(cell) => unsafe { &cell.get().as_ref::<'a>().unwrap().1 },
            None => &None,
        }
    }
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        if let Some(lock) = self.value_lock {
            unsafe { Box::from_raw(lock)};
        }
    }
}

pub struct RefMut<'a, T> {
    map_lock: RwLockReadGuard<'a, TheMap<T>>,
    value_lock: *mut cell::RefMut<'a, ()>,

    // The following note and property are modified from 
    // https://github.com/kinghajj/deque/blob/master/src/lib.rs#L67
    //
    // Marker so that the Ref is not Sync. The Ref can only be
    // accessed from a single thread at once. Ideally we would use a negative
    // impl here but these are not stable yet.
    phantom: PhantomData<Cell<()>>,
}

impl<'a, T> RefMut<'a, T> {
    fn new(lock: RwLockReadGuard<'a, TheMap<T>>) -> Self {
        Self { 
            value_lock: Box::into_raw(Box::new(unsafe { (*lock.get(&thread::current().id()).unwrap().get()).0.borrow_mut() })), 
            map_lock: lock, 
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = Option<T>;

    fn deref(&self) -> &'a Self::Target {
        match self.map_lock.get(&thread::current().id()) {
            Some(cell) => unsafe { &cell.get().as_ref::<'a>().unwrap().1 },
            None => &None,
        }
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe { &mut self.map_lock.get(&thread::current().id()).unwrap().get().as_mut::<'a>().unwrap().1 }
    }
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.value_lock)};
    }
}

