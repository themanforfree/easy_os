use alloc::sync::{Arc, Weak};

use crate::{fs::File, memory::UserBuffer, proc::suspend_current_and_run_next, sync::UPSafeCell};

const RING_BUFFER_SIZE: usize = 32;

pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Arc<UPSafeCell<PipeRingBuffer>>,
}

pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize, // point to the next write position
    tail: usize, // point to the next read position
    write_end: Option<Weak<Pipe>>,
}

impl Pipe {
    pub fn read_end_with_buffer(buffer: Arc<UPSafeCell<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    pub fn write_end_with_buffer(buffer: Arc<UPSafeCell<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer,
        }
    }

    pub fn new() -> (Arc<Pipe>, Arc<Pipe>) {
        let buffer = Arc::new(unsafe { UPSafeCell::new(PipeRingBuffer::new()) });
        let read_end = Arc::new(Pipe::read_end_with_buffer(buffer.clone()));
        let write_end = Arc::new(Pipe::write_end_with_buffer(buffer.clone()));
        buffer.borrow_mut().set_write_end(&write_end);
        (read_end, write_end)
    }
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            write_end: None,
        }
    }

    fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        self.write_end = Some(Arc::downgrade(write_end));
    }

    fn is_full(&self) -> bool {
        (self.head + 1) % RING_BUFFER_SIZE == self.tail
    }

    fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    fn push(&mut self, byte: u8) -> Result<(), u8> {
        if self.is_full() {
            return Err(byte);
        }
        self.arr[self.head] = byte;
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        Ok(())
    }

    fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        let byte = self.arr[self.tail];
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        Some(byte)
    }

    fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
}

impl File for Pipe {
    fn readable(&self) -> bool {
        self.readable
    }

    fn writable(&self) -> bool {
        self.writable
    }

    /// Reads data until the buffer is full or no more data is available.
    fn read(&self, buf: UserBuffer) -> usize {
        assert!(self.readable());
        let mut buf_iter = buf.into_iter();
        let mut read_cnt = 0;
        loop {
            let mut ring_buffer = self.buffer.borrow_mut();
            if ring_buffer.is_empty() {
                if ring_buffer.all_write_ends_closed() {
                    return read_cnt; // No more data to read
                }
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue; // Retry reading after suspension
            }
            let Some(byte_ptr) = buf_iter.next() else {
                return read_cnt; // Buffer is full or no more data to read
            };
            unsafe { *byte_ptr = ring_buffer.pop().unwrap() };
            read_cnt += 1;
        }
    }

    /// Writes data until the buffer is full or no more data needs to be written.
    fn write(&self, buf: UserBuffer) -> usize {
        assert!(self.writable());
        let mut buf_iter = buf.into_iter();
        let mut write_cnt = 0;
        loop {
            let mut ring_buffer = self.buffer.borrow_mut();
            if ring_buffer.is_full() {
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            let Some(byte_ptr) = buf_iter.next() else {
                return write_cnt;
            };
            ring_buffer.push(unsafe { *byte_ptr }).unwrap();
            write_cnt += 1;
        }
    }
}
