use ostd::mm::{Fallible, Frame, FrameAllocOptions, HasPaddr, VmReader, PAGE_SIZE};
use sbi_rt::Physical;
use spin::Once;

static RECEIVE_BUFFER: Once<Frame> = Once::new();

pub fn receive_str<F>(mut callback: F) -> usize
where
    F: FnMut(VmReader<Fallible>),
{
    if !RECEIVE_BUFFER.is_completed() {
        RECEIVE_BUFFER.call_once(|| FrameAllocOptions::new(1).alloc().unwrap().pop().unwrap());
    }

    let paddr = RECEIVE_BUFFER.get().unwrap().paddr();
    let ret = sbi_rt::console_read(Physical::new(
        PAGE_SIZE,
        paddr & 0xFFFF_FFFF,
        (paddr >> 32) & 0xFFFF_FFFF,
    ));

    if ret.is_err() {
        return 0;
    }

    let read_bytes = ret.value;

    let reader: VmReader<'_, ostd::mm::Fallible> = RECEIVE_BUFFER
        .get()
        .unwrap()
        .reader()
        .limit(read_bytes)
        .to_fallible();

    callback.call_mut((reader,));
    read_bytes
}
