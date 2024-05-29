#![no_std]

use num_enum::FromPrimitive;

pub mod macros;

#[repr(usize)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum Syscall {
    Read = 0,
    Write = 1,

    GetPid = 39,

    Fork = 58,
    Spawn = 59,
    Exit = 60,
    WaitPid = 61,
    Kill = 62,
    Sem = 63,

    Time = 201,

    ListApp = 65529,
    Stat = 65530,
    Allocate = 65533,
    Deallocate = 65534,

    #[num_enum(default)]
    None = 65535,
}
