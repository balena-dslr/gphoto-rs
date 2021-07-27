#![no_std]

use libc_print::libc_println;

fn main() {
    let version = gphoto::libgphoto2_version();
    libc_println!(
        "libgphoto2 {} {} {} {} {}",
        version.version(),
        version.camlibs(),
        version.compiler(),
        version.ltdl(),
        version.exif()
    );

    let camera = match gphoto::Camera::autodetect() {
        Ok(c) => c,
        Err(err) => panic!("error opening camera: {}", err),
    };

    {
        let port = camera.port();

        libc_println!("[port info]");
        libc_println!("port type = {:?}", port.port_type());
        libc_println!("port name = {:?}", port.name());
        libc_println!("port path = {:?}", port.path());
    }
}
