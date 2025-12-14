use std::io;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("img/hello_work.ico")
            .compile()?;
    }
    Ok(())
}
