use std::{
    fs::{create_dir, read_dir, remove_file, OpenOptions},
    io::Write,
    process::Command,
};

fn main() {
    // Read the file inside the "user" directory.
    let mut user_program_names = Vec::new();
    let dir = read_dir("user").unwrap();
    for entry in dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();
            println!("cargo:rerun-if-changed=./user/{}", name);
            user_program_names.push(name);
        }
    }

    let _ = create_dir("./target/user_prog/");
    // Compile user mode program, if riscv toolchain does not exists, then ignore it.
    let path = find_gcc_path();
    if let Ok(path) = path {
        for file in user_program_names.iter_mut() {
            let mut gcc = Command::new(path.clone());
            gcc.arg("-static")
                .arg("-O2")
                .arg("-mabi=lp64")
                .arg(format!("./user/{}", file))
                .arg("-o");
            // Just ignore the stupid code here, it is just for convenience to remove ".c"
            file.pop();
            file.pop();
            gcc.arg(format!("./target/user_prog/{}", file));
            let val = gcc.spawn().unwrap().wait_with_output().unwrap();
            if !val.status.success() {
                panic!("Compile error in c. Output: {:?}", val);
            }
        }
    } else {
        for file in user_program_names.iter_mut() {
            // Just ignore the stupid code here, it is just for convenience to remove ".c"
            file.pop();
            file.pop();
        }
    }

    // Finally, spawn the src/fs/progs.rs file, with name-bytes array.
    // Template:
    // const {name_upper}: &[u8] =
    //     include_bytes_aligned::include_bytes_aligned!(32, "../../target/user_prog/{name}");
    //
    // pub fn init() {
    //     super::USER_PROGS.call_once(|| {
    //         let mut user_progs = alloc::collections::btree_map::BTreeMap::new();
    //         user_progs.insert("{name}", {name_upper}));
    //         user_progs
    //     });
    // }

    let _ = remove_file("./src/fs/progs.rs");
    let mut file = OpenOptions::new()
        .append(false)
        .create(true)
        .write(true)
        .open("./src/fs/progs.rs")
        .unwrap();

    for name in user_program_names.iter() {
        file.write_fmt(format_args!(
            "const {}: &[u8] =\n    include_bytes_aligned::include_bytes_aligned!(32, \"../../target/user_prog/{}\");\n",
            name.to_uppercase(),
            name,
        ))
        .unwrap();
    }

    file.write_all(b"\npub fn init() {\n    super::USER_PROGS.call_once(|| {\n        let mut user_progs = alloc::collections::btree_map::BTreeMap::new();\n").unwrap();

    for name in user_program_names {
        file.write_fmt(format_args!(
            "        user_progs.insert(\"{}\", {});\n",
            name,
            name.to_uppercase()
        ))
        .unwrap();
    }

    file.write_all(b"        user_progs\n    });\n}\n").unwrap();
    file.flush().unwrap();
}

/// Find the riscv64 gcc path in these path:
/// 1. riscv64-unknown-linux-gnu-gcc
/// 2. /root/riscv64-toolchain/riscv64-unknown-linux-gnu-gcc
/// 3. /opt/riscv64/bin/riscv64/riscv64-unknown-linux-gnu-gcc
fn find_gcc_path() -> Result<String, ()> {
    let path_list = [
        "riscv64-unknown-linux-gnu-gcc",
        "/root/riscv64-toolchain/riscv64-unknown-linux-gnu-gcc",
        "/opt/riscv64/bin/riscv64/riscv64-unknown-linux-gnu-gcc",
    ];

    for path in path_list {
        let mut command = Command::new(path);
        if command.spawn().is_ok() {
            return Ok(path.to_string());
        }
    }

    Err(())
}
