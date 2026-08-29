#[cfg(windows)]
fn main() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/app.ico");
    res.set("ProductName", "LarusX");
    res.set("FileDescription", "LarusX Competitive Display Tuner & Crosshair Suite");
    res.set("CompanyName", "Syojhin & Lara");
    res.set("LegalCopyright", "Copyright (c) 2026 Syojhin");
    let _ = res.compile();
}

#[cfg(not(windows))]
fn main() {}
