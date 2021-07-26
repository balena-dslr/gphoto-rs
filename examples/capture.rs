use std::path::Path;

fn main() {
    // open camera

    println!("opening camera ...");
    let mut camera = match gphoto::Camera::autodetect() {
        Ok(c) => c,
        Err(err) => panic!("error opening camera: {}", err),
    };
    println!(" (done)");

    // capture image

    println!("capturing image ...");
    let capture = match camera.capture_image() {
        Ok(c) => c,
        Err(err) => panic!("error capturing image: {}", err),
    };
    println!(" (done) {:?}", capture.basename());

    // download file

    let mut file = match gphoto::FileMedia::create(Path::new(capture.basename().as_ref())) {
        Ok(f) => f,
        Err(err) => panic!("error saving file: {}", err),
    };

    println!("downloading ...");
    if let Err(err) = camera.download(&capture, &mut file, None) {
        panic!("error downloading file: {}", err);
    }
    println!(" (done)");
}
