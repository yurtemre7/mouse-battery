fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo.ico");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource icon: {}", e);
        }
    }
}
