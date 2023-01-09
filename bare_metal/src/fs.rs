use alloc::vec;
use alloc::vec::Vec;
use fatfs::{NullTimeProvider, LossyOemCpConverter};
use crate::{block::DEVICE, sbi::shutdown};
#[derive(Debug)]
struct AtaError;

#[derive(Debug)]
enum DiskCursorIoError {
    UnexpectedEof,
    WriteZero,
}
impl fatfs::IoError for DiskCursorIoError {
    fn is_interrupted(&self) -> bool {
        false
    }

    fn new_unexpected_eof_error() -> Self {
        Self::UnexpectedEof
    }

    fn new_write_zero_error() -> Self {
        Self::WriteZero
    }
}

struct DiskCursor {
    sector: u64,
    offset: usize,
}

impl DiskCursor {
    fn get_position(&self) -> usize {
        (self.sector * 0x200) as usize + self.offset
    }

    fn set_position(&mut self, position: usize) {
        self.sector = (position / 0x200) as u64;
        self.offset = position % 0x200;
    }

    fn move_cursor(&mut self, amount: usize) {
        self.set_position(self.get_position() + amount)
    }
}

impl fatfs::IoBase for DiskCursor {
    type Error = DiskCursorIoError;
}

impl fatfs::Read for DiskCursor {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DiskCursorIoError> {
        // 由于读取扇区内容还需要考虑跨 cluster，因此 read 函数只读取一个扇区
        // 防止读取较多数据时超出限制
        // 读取所有的数据的功能交给 read_exact 来实现

        println!("read: buf_size: {}", buf.len());

        // 获取硬盘设备读取器（驱动？）
        let mut block_device = unsafe { DEVICE.get().unwrap() }.lock();

        // 如果 start 不是 0 或者 len 不是 512
        let read_size = if self.offset != 0 || buf.len() < 512 {
            let mut data = [0u8; 512];
            block_device.read_block(self.sector as usize, &mut data);

            let start = self.offset;
            let end = (self.offset + buf.len()).min(512);

            println!("sector: {}, start: {}, end: {}, data: \n{:?}", 
                self.sector, 
                start,
                end,
                &data[start..end]
            );
            if self.sector == 0 && start == 510 && end == 512 {
                println!("{:?}", &data);
            }

            if self.sector == 0 && start == 0 {
                println!("{:?}", &data);
                block_device.read_block(self.sector as usize + 1, &mut data);
                println!("{:?}", &data);
                shutdown();
            }

            buf.copy_from_slice(&data[start..end]);
            end-start
        } else {
            block_device.read_block(self.sector as usize, &mut buf[0..512]);
            512
        };

        self.move_cursor(read_size);
        Ok(read_size)
    }
}

impl fatfs::Write for DiskCursor {
    fn write(&mut self, buf: &[u8]) -> Result<usize, DiskCursorIoError> {
        // 由于写入扇区还需要考虑申请 cluster，因此 write 函数只写入一个扇区
        // 防止写入较多数据时超出限制
        // 写入所有的数据的功能交给 write_all 来实现

        // 获取硬盘设备写入器（驱动？）
        let mut block_device = unsafe { DEVICE.get_mut().unwrap() }.lock();

        // 如果 start 不是 0 或者 len 不是 512
        let write_size = if self.offset != 0 || buf.len() < 512 {
            let mut data = [0u8; 512];
            block_device.read_block(self.sector as usize, &mut data);

            let start = self.offset;
            let end = (self.offset + buf.len()).min(512);
            
            data[start..end].clone_from_slice(&buf);
            block_device.write_block(self.sector as usize, &mut data);

            end-start
        } else {
            block_device.write_block(self.sector as usize, &buf[0..512]);
            512
        };

        self.move_cursor(write_size);
        Ok(write_size)
    }

    fn flush(&mut self) -> Result<(), DiskCursorIoError> {
        Ok(())
    }
}

impl fatfs::Seek for DiskCursor {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, DiskCursorIoError> {
        match pos {
            fatfs::SeekFrom::Start(i) => {
                self.set_position(i as usize);
                Ok(i)
            }
            fatfs::SeekFrom::End(i) => {
                todo!("Seek from end")
            }
            fatfs::SeekFrom::Current(i) => {
                let new_pos = (self.get_position() as i64) + i;
                self.set_position(new_pos as usize);
                Ok(new_pos as u64)
            }
        }
    }
}

type Dir<'a> = fatfs::Dir<'a, DiskCursor, NullTimeProvider, LossyOemCpConverter>;

fn show_files(dir: Dir, space: usize) {
    println!("show files");
    for file in dir.iter() {
        // if let Ok(file) = file {
        //     println!("{}", file.file_name());
        // }
    }
}

pub fn ls_dir(path: &str) {
    let c = DiskCursor {
        sector: 0,
        offset: 0,
    };

    // 获取文件
    let fs = fatfs::FileSystem::new(c, fatfs::FsOptions::new()).expect("open fs fai");
    println!("Hello");
    let mut cursor =fs.root_dir();
    show_files(cursor, 0);

    // let mut cursor = fs.root_dir();
    // let mut file;
    // if let Ok(file1) = cursor.open_file("test4.txt") {
    //     file = file1;
    // } else {
    //     file = cursor.create_file("test4.txt").expect("can't create file");
    // };

    

    // // 写入文件 使用 write_all
    // file.seek(SeekFrom::End(0));
    
    // for _ in 0..1 {
    //     file.write_all(b"Hello nvme\n").expect("can't write file");
    // }
    // file.flush().expect("fail flush");
    
    // // 读取文件 使用 read_exact
    // let file_size = file.seek(SeekFrom::End(0)).expect("can't seek file");
    // file.seek(SeekFrom::Start(0)).expect("can't seek file to start");
    // let mut data_buf = vec![0u8; file_size as usize];
    // file.read_exact(&mut data_buf).expect("can't read file");
    // _puts(&data_buf);
}