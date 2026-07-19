use std::time::SystemTime;

use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
    pub struct FilePermissions: u16 {
        /// set-user-ID bit (see execve(2))
        const SUID = 0o4000;
        /// set-group-ID bit (see below)
        const SGID = 0o2000;
        /// sticky bit (see below)
        const SVTX = 0o1000;
        /// owner has read, write, and execute permission
        const RWXU = 0o0700;
        /// owner has read permission
        const RUSR = 0o0400;
        /// owner has write permission
        const WUSR = 0o0200;
        /// owner has execute permission
        const XUSR = 0o0100;
        /// group has read, write, and execute permission
        const RWXG = 0o0070;
        /// group has read permission
        const RGRP = 0o0040;
        /// group has write permission
        const WGRP = 0o0020;
        /// group has execute permission
        const XGRP = 0o0010;
        /// others (not in group) have read, write, and execute permission
        const RWXO = 0o0007;
        /// others have read permission
        const ROTH = 0o0004;
        /// others have write permission
        const WOTH = 0o0002;
        /// others have execute permission
        const XOTH = 0o0001;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub atime: SystemTime,
    pub btime: SystemTime,
    pub ctime: SystemTime,
    pub mtime: SystemTime,

    pub permissions: FilePermissions,
    pub file_type: FileType,

    pub uid: u32,
    pub gid: u32,

    // pub attributes: StatxAttributes, // TODO
    // pub attributes_mask: StatxAttributes, // TODO
    pub blksize: u32,
    pub nlink: u32,
    pub ino: u64,
    pub size: u64,
    pub blocks: u64,
    pub rdev_major: u32,
    pub rdev_minor: u32,
    pub dev_major: u32,
    pub dev_minor: u32,
    pub mnt_id: u64,
    pub subvol: u64,
    pub atomic_write_unit_min: u32,
    pub atomic_write_unit_max: u32,
    pub atomic_write_unit_max_opt: u32,
    pub atomic_write_segments_max: u32,
    pub dio_mem_align: u32,
    pub dio_offset_align: u32,
    pub dio_read_offset_align: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    BlockDevice,
    CharacterDevice,
    Directory,
    Fifo,
    Symlink,
    Regular,
    Socket,
    Unknown,
}
