use std::{collections::HashMap, thread::{ThreadId, self}, cell::{UnsafeCell, Cell}, sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}, ops::{Deref, DerefMut}, marker::PhantomData};

// TODO: Implement my own HashMap that makes no assumptions about what is being mutated when
// and can be used without the RwLock

#[cfg(test)]
mod tests;

type TheMap<T> = HashMap<ThreadId, UnsafeCell<Option<T>>>;

pub struct ByThreadCell<T> {
    value: RwLock<TheMap<T>>,
}

unsafe impl<T> Sync for ByThreadCell<T> { }

impl<T> ByThreadCell<T> {
    pub fn new() -> Self {
        Self { value: RwLock::new(TheMap::new()) }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        Ref { lock: self.value.read().unwrap(), phantom: PhantomData }
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        RefMut { lock: self.value.write().unwrap(), phantom: PhantomData }
    }
}

pub struct Ref<'a, T> {
    lock: RwLockReadGuard<'a, TheMap<T>>,

    // The following note and property are modified from 
    // https://github.com/kinghajj/deque/blob/master/src/lib.rs#L67
    //
    // Marker so that the Ref is not Sync. The Ref can only be
    // accessed from a single thread at once. Ideally we would use a negative
    // impl here but these are not stable yet.
    phantom: PhantomData<Cell<()>>,
}

macro_rules! deref_impl {
    ($($t:ident),+) => {
        $(
            impl<'a, T> Deref for $t<'a, T> {
                type Target = Option<T>;
            
                fn deref(&self) -> &'a Self::Target {
                    match self.lock.get(&thread::current().id()) {
                        Some(cell) => unsafe { cell.get().as_ref::<'a>().unwrap() },
                        None => &None,
                    }
                }
            }
        )+
    };
}

deref_impl!(Ref, RefMut);

pub struct RefMut<'a, T> {
    lock: RwLockWriteGuard<'a, TheMap<T>>,

    // The following note and property are modified from 
    // https://github.com/kinghajj/deque/blob/master/src/lib.rs#L67
    //
    // Marker so that the Ref is not Sync. The Ref can only be
    // accessed from a single thread at once. Ideally we would use a negative
    // impl here but these are not stable yet.
    phantom: PhantomData<Cell<()>>,
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe { self.lock.entry(thread::current().id()).or_insert(UnsafeCell::new(None)).get().as_mut::<'a>().unwrap() }
    }
}