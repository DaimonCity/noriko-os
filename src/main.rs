fn main() {
    let bios_path = env!("BIOS_PATH");
    let status = std::process::Command::new("qemu-system-x86_64")
        .args(["-drive", &format!("format=raw,file={bios_path}")])
        .args(["-serial", "stdio"])
        .status()
        .unwrap();
    std::process::exit(status.code().unwrap_or(1));
}