fn main() {
    use std::{env, fs, path::PathBuf};

    let ld = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("linker.ld");
    println!("{}", ld.display());
    fs::write(&ld, LINKER).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-arg=-T{}", ld.display());
    println!("cargo:rustc-force-frame-pointers=yes");
}

const LINKER: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80400000;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        ebss = .;
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}";
