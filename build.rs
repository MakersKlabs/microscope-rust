//! On Windows, embed the application icon and version info into the executable.

fn main() {
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icon.ico");
        resource.set("FileDescription", "Microscope");
        resource.set("ProductName", "Microscope");
        resource.set("LegalCopyright", "MIT License");
        let _ = resource.compile();
    }
}
