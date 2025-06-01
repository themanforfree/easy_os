use std::{
    env,
    fs::{self, File},
    io::{Result, Write},
    path::PathBuf,
};

fn main() {
    let ld = PathBuf::from(env::var("OUT_DIR").unwrap()).join("linker.ld");
    println!("{}", ld.display());
    fs::write(&ld, LINKER).unwrap();
    insert_app_data().unwrap();
    println!("cargo:rerun-if-changed=../user_lib");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LOG");
    println!("cargo:rustc-link-arg=-T{}", ld.display());
    println!("cargo:rustc-force-frame-pointers=yes");
}

fn insert_app_data() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap();
    let target_path = out_dir.ancestors().nth(4).unwrap();

    let mut f = File::create("src/link_app.S")?;
    // let apps: Vec<_> = read_dir("../user_lib/src/bin")
    //     .unwrap()
    //     .map(|dir_entry| {
    //         let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
    //         name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
    //         name_with_ext
    //     })
    //     .collect();
    let apps = ["init"];

    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_apps
_num_apps:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{i}_start"#)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    writeln!(
        f,
        r#"
    .global _app_names
_app_names:"#
    )?;
    for app in apps.iter() {
        writeln!(f, r#"    .string "{app}""#)?;
    }

    for (idx, app) in apps.iter().enumerate() {
        let elf_path = target_path.join(&profile).join(app);
        writeln!(
            f,
            r#"
    .section .data
    .global app_{idx}_start
    .global app_{idx}_end
    .align 3
app_{idx}_start:
    .incbin "{path}"
app_{idx}_end:"#,
            idx = idx,
            path = elf_path.display(),
        )?;
    }
    Ok(())
}

const LINKER: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline);
        . = ALIGN(4K);
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    sbss_with_stack = .;
    .bss : {
        *(.bss.uninit)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}";
