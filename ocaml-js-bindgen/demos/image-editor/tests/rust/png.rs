    #[test]
    fn png_dimensions_and_truncated_streams_are_bounded() {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, super::MAX_DIMENSION + 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&vec![0; (super::MAX_DIMENSION as usize + 1) * 4])
                .unwrap();
        }
        assert!(matches!(
            super::open_png(&bytes),
            Err(super::ImageError::TooLarge)
        ));
        assert!(matches!(
            super::open_png(b"invalid PNG"),
            Err(super::ImageError::InvalidPng)
        ));
        assert!(matches!(
            super::open_png(&include_bytes!("../fixture.png")[..40]),
            Err(super::ImageError::InvalidPng)
        ));
    }

#[test]
fn rgba_and_rgb_pngs_decode_to_the_same_pixels() {
    for (color, source) in [(png::ColorType::Rgba, vec![1, 2, 3, 255]), (png::ColorType::Rgb, vec![1, 2, 3])] {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
            encoder.set_color(color);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header().unwrap().write_image_data(&source).unwrap();
        }
        assert_eq!(super::open_png(&bytes).ok().unwrap().pixels, [1, 2, 3, 255]);
    }
}
