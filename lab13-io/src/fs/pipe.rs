use crate::error::{Errno, Error, Result};
use crate::fs::FileLike;
use alloc::sync::Arc;
use ostd::mm::{FrameAllocOptions, PAGE_SIZE, Segment, VmIo, VmReader, VmWriter};
use spin::Mutex;

pub struct PipeReader {
    pipe: Arc<Pipe>,
}

pub struct PipeWriter {
    pipe: Arc<Pipe>,
}

pub struct Pipe {
    buffer: Segment<()>,
    inner: Mutex<Inner>,
}

struct Inner {
    pos: usize,
    current_size: usize,
}

const DEFAULT_PIPE_BUF_SIZE: usize = 65536;

impl Pipe {
    pub fn new_pair() -> (Arc<PipeReader>, Arc<PipeWriter>) {
        let buffer = FrameAllocOptions::new()
            .alloc_segment(DEFAULT_PIPE_BUF_SIZE / PAGE_SIZE)
            .unwrap();

        let pipe = Arc::new(Self {
            buffer,
            inner: Mutex::new(Inner {
                pos: 0,
                current_size: 0,
            }),
        });

        let reader = Arc::new(PipeReader { pipe: pipe.clone() });
        let writer = Arc::new(PipeWriter { pipe });

        (reader, writer)
    }
}

impl FileLike for PipeWriter {
    fn read(&self, _writer: VmWriter) -> Result<usize> {
        Err(Error::new(Errno::EBADF))
    }

    fn write(&self, mut reader: VmReader) -> Result<usize> {
        let mut total_written = 0;
        let mut inner = self.pipe.inner.lock();

        loop {
            let current_size = inner.current_size;
            if current_size >= DEFAULT_PIPE_BUF_SIZE {
                break;
            }

            let write_pos = (inner.pos + current_size) % DEFAULT_PIPE_BUF_SIZE;

            let to_write = core::cmp::min(reader.remain(), DEFAULT_PIPE_BUF_SIZE - current_size);
            if to_write == 0 {
                break;
            }

            let buffer_offset = write_pos % DEFAULT_PIPE_BUF_SIZE;
            let first_chunk = core::cmp::min(to_write, DEFAULT_PIPE_BUF_SIZE - buffer_offset);
            let second_chunk = to_write - first_chunk;

            // Write first chunk
            self.pipe.buffer.write(buffer_offset, &mut reader).unwrap();
            total_written += first_chunk;

            // Write second chunk if needed
            if second_chunk > 0 {
                self.pipe.buffer.write(0, &mut reader).unwrap();
                total_written += second_chunk;
            }

            inner.current_size += to_write;
        }

        Ok(total_written)
    }
}

impl FileLike for PipeReader {
    fn read(&self, mut writer: VmWriter) -> Result<usize> {
        let mut total_read = 0;
        let mut inner = Some(self.pipe.inner.lock());

        let mut inner = inner.take().unwrap();

        loop {
            let current_size = inner.current_size;

            if current_size == 0 {
                break;
            }

            let read_pos = inner.pos % DEFAULT_PIPE_BUF_SIZE;

            let to_read = core::cmp::min(writer.avail(), current_size);
            if to_read == 0 {
                break;
            }

            let buffer_offset = read_pos % DEFAULT_PIPE_BUF_SIZE;
            let first_chunk = core::cmp::min(to_read, DEFAULT_PIPE_BUF_SIZE - buffer_offset);
            let second_chunk = to_read - first_chunk;

            // Read first chunk
            self.pipe.buffer.read(buffer_offset, &mut writer).unwrap();
            total_read += first_chunk;

            // Read second chunk if needed
            if second_chunk > 0 {
                self.pipe.buffer.read(0, &mut writer).unwrap();
                total_read += second_chunk;
            }

            inner.current_size -= to_read;
        }

        Ok(total_read)
    }

    fn write(&self, _reader: VmReader) -> Result<usize> {
        Err(Error::new(Errno::EBADF))
    }
}
