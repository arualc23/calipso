use std::{cell::UnsafeCell, fmt::Debug, sync::{Arc, atomic::{AtomicU8, Ordering}}};

/// Double buffer implementation that can be shared across two threads. 

#[derive(Clone, Copy)]
enum Flag {
    Free,
    Reading,
    Swapping,
}
impl Flag {
    const fn as_u8(self) -> u8 {
        match self {
            Self::Free => 0,
            Self::Reading => 1,
            Self::Swapping => 2,
        }
    }
}

/// # Safety
/// All safety mechanism are implemented in [DBufferReader] and [DBufferWriter]. Atomic flag is used to ensure that only one of the two threads 
/// can access the reading buffer at a time. The writing buffer will only ever be accessed by the writer.
struct DoubleBuffer<T: Clone> {
    write: UnsafeCell<T>,
    read: UnsafeCell<T>,
    flag: AtomicU8
}

impl<T: Clone> DoubleBuffer<T> {
    /// # Safety
    /// Caller must ensure that there are no other references to the underlaying data i. e. nothing is reading or writing now.
    unsafe fn swap(&self) {
        unsafe { std::mem::swap(self.read.get().as_mut_unchecked(), self.write.get().as_mut_unchecked()); }
        // std::mem::swap(&mut self.read, &mut self.write);
    }

    /// # Safety
    /// Caller must ensure that there are no other references to the underlaying data i. e. nothing is reading or writing now.
    #[inline]
    unsafe fn write(&self, data: T) {
        unsafe { *self.write.get() = data; }
        
    }

    /// # Safety
    /// Caller must ensure that there are no other references to the underlaying data i. e. nothing is reading or writing now.
    #[inline]
    unsafe fn read(&self) -> T {
        unsafe { (*self.read.get()).clone() }
    }
}

#[inline]
fn block_on_flag(flag: &AtomicU8, current: Flag, new: Flag) {
    while let Err(_) = flag.compare_exchange(current.as_u8(), new.as_u8(), Ordering::AcqRel, Ordering::Relaxed) {
        std::hint::spin_loop();
    }
}

///Writer of the double buffer. [Self::write] writes to the buffer, then blocks and swaps safely.
pub struct DBufferWriter<T: Clone> {
    inner: Arc<DoubleBuffer<T>>
}

///Reader of the double buffer.
pub struct DBufferReader<T: Clone> {
    inner: Arc<DoubleBuffer<T>>
}

impl<T: Clone> DBufferReader<T> {
    ///Clones the value out of the buffer.
    pub fn read(&mut self) -> T {
        block_on_flag(&self.inner.flag, Flag::Free, Flag::Reading);
        //Safety: we checked the swapping flag, and we set the reading flag
        let res = unsafe {
            (*self.inner).read()
        };

        block_on_flag(&self.inner.flag, Flag::Reading, Flag::Free);
        res
    }
}

impl<T: Clone + Debug> DBufferWriter<T> {
    pub fn write(&mut self, data: T) {
        //Safety: there is no other access to the writing buffer.
        unsafe {
            self.inner.write(data);
        }

        self.swap();
        
    }

    fn swap(&mut self) {
        block_on_flag(&self.inner.flag, Flag::Free, Flag::Swapping);

        
        //Safety: we checked the reading flag, and set the swapping flag.
        unsafe {
            // log::info!("Obtained lock, swapping. Data in write buffer: {:?}", *self.inner.write.get());
            self.inner.swap();
        }

        block_on_flag(&self.inner.flag, Flag::Swapping, Flag::Free);
    }
}

unsafe impl<T: Send + 'static + Clone> Send for DBufferWriter<T> {}
unsafe impl<T: Send + 'static + Clone> Send for DBufferReader<T> {}

pub(crate) fn new<T: Clone>(init: T) -> (DBufferWriter<T>, DBufferReader<T>) {
    let dbuffer = DoubleBuffer {
        write: UnsafeCell::new(init.clone()),
        read: UnsafeCell::new(init),
        flag: AtomicU8::new(Flag::Free.as_u8())
    };

    let arc = Arc::new(dbuffer);

    let writer = DBufferWriter {inner: arc.clone()};
    let reader = DBufferReader {inner: arc};

    (writer, reader)

}