#![cfg_attr(not(unix), no_std)]
#![cfg_attr(not(unix), feature(custom_test_frameworks))]
#![cfg_attr(not(unix), test_runner(test_runner))]

extern crate alloc;

mod bitmap;
mod cache;
mod dev;
mod efs;
mod layout;
mod vfs;

pub use dev::BlockDevice;
pub use efs::EasyFileSystem;

/// Size of a block in bytes
const BLOCK_SIZE: usize = 512;
/// Use a block cache of 16 blocks
const BLOCK_CACHE_SIZE: usize = 16;

#[cfg(all(not(unix), test))]
fn test_runner(_tests: &[&dyn Fn()]) {
    unreachable!("this function will never be called");
}

#[cfg(all(unix, test))]
mod test {
    use super::*;
    use std::{
        fs::{File, OpenOptions},
        io::{Read, Seek, SeekFrom, Write},
        sync::{Arc, Mutex},
    };

    struct BlockFile(Mutex<File>);

    impl BlockDevice for BlockFile {
        fn read_block(&self, block_id: usize, buf: &mut [u8]) {
            let mut file = self.0.lock().unwrap();
            file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64))
                .expect("Error when seeking!");
            assert_eq!(file.read(buf).unwrap(), BLOCK_SIZE, "Not a complete block!");
        }

        fn write_block(&self, block_id: usize, buf: &[u8]) {
            let mut file = self.0.lock().unwrap();
            file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64))
                .expect("Error when seeking!");
            assert_eq!(
                file.write(buf).unwrap(),
                BLOCK_SIZE,
                "Not a complete block!"
            );
        }
    }

    #[test]
    fn test_fs() -> std::io::Result<()> {
        let block_file = Arc::new(BlockFile(Mutex::new({
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open("fs.img")?;
            f.set_len(8192 * 512).unwrap();
            f
        })));

        let efs = EasyFileSystem::create(block_file.clone(), 4096, 1);
        let root_inode = EasyFileSystem::root_inode(&efs);
        root_inode.create("file_a");
        root_inode.create("file_b");
        for name in root_inode.ls() {
            println!("{}", name);
        }
        let file_a = root_inode.find("file_a").unwrap();
        let greet_str = "Hello, world!";
        file_a.write_at(0, greet_str.as_bytes());
        //let mut buffer = [0u8; 512];
        let mut buffer = [0u8; 233];
        let len = file_a.read_at(0, &mut buffer);
        assert_eq!(greet_str, core::str::from_utf8(&buffer[..len]).unwrap(),);

        let mut random_str_test = |len: usize| {
            file_a.clear();
            assert_eq!(file_a.read_at(0, &mut buffer), 0,);
            let mut str = String::new();
            use rand;
            // random digit
            for _ in 0..len {
                str.push(char::from('0' as u8 + rand::random::<u8>() % 10));
            }
            file_a.write_at(0, str.as_bytes());
            let mut read_buffer = [0u8; 1024];
            let mut offset = 0usize;
            let mut read_str = String::new();
            loop {
                let len = file_a.read_at(offset, &mut read_buffer);
                if len == 0 {
                    break;
                }
                offset += len;
                read_str.push_str(core::str::from_utf8(&read_buffer[..len]).unwrap());
            }
            assert_eq!(str, read_str);
        };

        random_str_test(4 * BLOCK_SIZE);
        random_str_test(8 * BLOCK_SIZE + BLOCK_SIZE / 2);
        random_str_test(100 * BLOCK_SIZE);
        random_str_test(70 * BLOCK_SIZE + BLOCK_SIZE / 7);
        random_str_test((12 + 128) * BLOCK_SIZE);
        random_str_test(200 * BLOCK_SIZE);
        random_str_test(400 * BLOCK_SIZE);
        random_str_test(1000 * BLOCK_SIZE);
        random_str_test(2000 * BLOCK_SIZE);

        Ok(())
    }
}
