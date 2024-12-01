//! `Arc<Inode>` -> `OSInodeInner`: In order to open files concurrently
//! we need to wrap `Inode` into `Arc`,but `Mutex` in `Inode` prevents
//! file systems from being accessed simultaneously
//!
//! `UPSafeCell<OSInodeInner>` -> `OSInode`: for static `ROOT_INODE`,we
//! need to wrap `OSInodeInner` into `UPSafeCell`

use super::File;
use crate::drivers::BLOCK_DEVICE;
use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::*;
use easy_fs::{EasyFileSystem, Inode};
use lazy_static::*;
use crate::fs::{Stat,StatMode};

/// inode in memory
/// A wrapper around a filesystem inode
/// to implement File trait atop
pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}
/// The OS inode inner in 'UPSafeCell'
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

impl OSInode {
    /// create a new inode in memory
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    /// read all data from the inode
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(EasyFileSystem::root_inode(&efs))
    };
    pub static ref LINK_VEC: UPSafeCell<Vec<(Inode, u32)>> = unsafe {
        UPSafeCell::new(Vec::new())
    };
}

/// List all apps in the root directory
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/");
}

bitflags! {
    ///  The flags argument to the open() system call is constructed by ORing together zero or more of the following values:
    pub struct OpenFlags: u32 {
        /// readyonly
        const RDONLY = 0;
        /// writeonly
        const WRONLY = 1 << 0;
        /// read and write
        const RDWR = 1 << 1;
        /// create new file
        const CREATE = 1 << 9;
        /// truncate file size to 0
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

pub fn link(old: &Inode) {
    let mut link_list = LINK_VEC.exclusive_access();
    // 查找是否存在旧的 Inode
    let mut found = false;
    for (inode, count) in link_list.iter_mut() {
        if inode.get_inode_num() == old.get_inode_num() {
            // 如果找到，增加对应的 u32 值
            *count += 1;
            found = true;
            break;
        }
    }
    // 如果没有找到，创建新的 Inode 和 u32 元组，插入 vec
    if !found {
        link_list.push(((*old).clone(), 2));
    }
}
pub fn unlink(old: &Inode) -> bool{
    let mut link_vec = LINK_VEC.exclusive_access();  // 获取 LINK_VEC 的可变引用
    let mut find = false;
    for i in 0..link_vec.len() {
        if link_vec[i].0.get_inode_num() == old.get_inode_num() {
            let (_, count) = &mut link_vec[i];
            *count -= 1;
            // 如果引用计数为 0，移除该 inode
            if *count == 1 {
                link_vec.remove(i);  // 移除该元素
            }
            find = true;
            break;
        }
    }
    //println!("find:{}",find);
    find
}

/// Open a file
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = ROOT_INODE.find(name) {
            // clear size
            inode.clear();
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            ROOT_INODE
                .create(name)
                .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        ROOT_INODE.find(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

/*
///
pub fn delete_file(name: &str) -> isize{
    if let Some(inode) = ROOT_INODE.find(name) {
        // clear size
        inode.clear();
    } else {

    }
    0
}
 */
///
pub fn create_link(old_name: &str, new_name: &str) -> isize{
    let old_inode = ROOT_INODE.find(old_name);
    //let new_inode = ROOT_INODE.find(new_name);
    match old_inode {
        Some(mut old) => {
            let old_inode_id = old.get_inode_num();
            let new_inode = ROOT_INODE.create_link(new_name,old_inode_id);
            match new_inode {
                Some(mut new) =>{
                    link(&old);
                    // 尝试通过 Arc::get_mut() 获取可变引用
                    if let Some(old_mut) = Arc::get_mut(&mut old) {
                        if let Some(new_mut) = Arc::get_mut(&mut new) {
                            old_mut.build_link();
                            new_mut.build_link();
                            //println!("old.link:{}",old.get_link());
                            //println!("new.link:{}",new.get_link());
                        } else {
                            return -1;
                        }
                    }
                },
                None => return -1,
            }
            
        },
        None => return -1,
    }
    0
}

///
pub fn destroy_link(name: &str) -> isize{
    let find_inode = ROOT_INODE.find(name);
    //let new_inode = ROOT_INODE.find(new_name);
    match find_inode {
        Some(mut inode) => {
            let find = unlink(&inode);
            if let Some(mut_inode) = Arc::get_mut(&mut inode) {
                mut_inode.destroy_link();
                //println!("inode.link:{}",inode.get_link());
            }
            
            if !find {
                ROOT_INODE.remove_name_from_dir(name);
            }
        },
        None => return -1,
    }

    0
}

#[allow(unused)]
/// 打印 LINK_VEC 的内容
pub fn print_link_vec() {
    let link_vec = LINK_VEC.exclusive_access();
    println!("LINK_VEC contains:");
    // 遍历 LINK_VEC 中的每个 (Inode, u32) 元组
    for (old, count) in link_vec.iter() {
        println!("Old Inode: {:?}, Link Count: {}", old, count);
    }
}


#[allow(unused)]
/// 打印 VEC 的内容
pub fn print_vec(vec: &Vec<u32>) {
    println!("new_link_list contains:");
    for data in vec.iter() {
        println!("Inode_id: {}", data);
    }
}


impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
    fn fstat(&self) -> Option<Stat> {
        let inner = self.inner.exclusive_access();
        let inode_id = inner.inode.get_inode_num();

        let mut link_num: u32 = 1;
        //print_link_vec();
        //println!("inner.inode.get_link():{}",inner.inode.get_link());
        /* 
        if !inner.inode.get_link() {
            link_num = 1;
        }else{
            for (inode1,inode2) in link_list.iter(){
                if inode1.get_inode_num() == inode_id
                || inode2.get_inode_num() == inode_id{
                    link_num += 1;
                }
            }
        }*/
        let link_list =  LINK_VEC.exclusive_access();
        for (inode,count) in link_list.iter(){
            let inode_num = inode.get_inode_num();
            if inode_num == inode_id {
                link_num = *count;
            }
        }


        //let link = inner.inode.get_link();
        let stat_mode = match inner.inode.is_dir() {
            true => StatMode::DIR,
            false => StatMode::FILE,
        };
        Some(Stat {
            dev: 0,
            ino: inode_id as u64,
            mode: stat_mode,
            nlink: link_num,
            pad: [0; 7]
        })
    }
}
