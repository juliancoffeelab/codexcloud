    use super::*;
    fn at(p: &Paint, x: u32, y: u32) -> &[u8] {
        let i = ((y * p.image.width + x) * 4) as usize;
        &p.image.pixels[i..i + 4]
    }
    #[test]
    fn drawing_history_fill_cancel_and_png_roundtrip() {
        let mut p = create(32, 24).ok().unwrap();
        begin(&mut p, 2., 3., Tool::Brush, 0xff0000, 1);
        move_to(&mut p, 20., 3.);
        finish(&mut p);
        assert_eq!(at(&p, 12, 3), [255, 0, 0, 255]);
        assert_eq!(p.past.len(), 1);
        undo(&mut p);
        assert_eq!(at(&p, 12, 3), [255; 4]);
        redo(&mut p);
        assert_eq!(at(&p, 12, 3), [255, 0, 0, 255]);
        begin(&mut p, 10., 3., Tool::Eraser, 0, 1);
        finish(&mut p);
        assert_eq!(at(&p, 10, 3), [255; 4]);
        begin(&mut p, 0., 0., Tool::Fill, 0x0000ff, 1);
        finish(&mut p);
        assert_eq!(at(&p, 0, 0), [0, 0, 255, 255]);
        assert_eq!(at(&p, 12, 3), [255, 0, 0, 255]);
        begin(&mut p, 4., 4., Tool::Rectangle, 0, 1);
        move_to(&mut p, 15., 16.);
        cancel(&mut p);
        assert_eq!(at(&p, 4, 4), [0, 0, 255, 255]);
        let png = png_bytes(&p).ok().unwrap();
        assert_eq!(open_png(&png).ok().unwrap().pixels, p.image.pixels);
        undo(&mut p);
        clear(&mut p);
        assert!(p.future.is_empty());
    }
    #[test]
    fn selection_moves_resizes_and_cancels_from_original_pixels() {
        let mut p = create(16, 16).ok().unwrap();
        begin(&mut p, 2., 2., Tool::Rectangle, 0xff0000, 1);
        move_to(&mut p, 5., 5.);
        finish(&mut p);
        let original = p.image.pixels.clone();
        let source = Region {
            x: 2,
            y: 2,
            width: 4,
            height: 4,
        };
        assert!(transform_start(&mut p, source).is_ok());
        assert!(
            transform_to(
                &mut p,
                Region {
                    x: 4,
                    y: 2,
                    ..source
                }
            )
            .is_ok()
        );
        assert_eq!(at(&p, 4, 2), [255, 0, 0, 255]);
        assert_eq!(at(&p, 2, 2), [255; 4]);
        assert!(
            transform_to(
                &mut p,
                Region {
                    x: 6,
                    y: 6,
                    width: 8,
                    height: 8
                }
            )
            .is_ok()
        );
        assert_eq!(at(&p, 6, 6), [255, 0, 0, 255]);
        assert!(
            transform_to(
                &mut p,
                Region {
                    x: 15,
                    y: 0,
                    width: 8,
                    height: 8
                }
            )
            .is_err()
        );
        cancel(&mut p);
        assert_eq!(p.image.pixels, original);
        assert!(transform_start(&mut p, source).is_ok());
        assert!(
            transform_to(
                &mut p,
                Region {
                    x: 6,
                    y: 6,
                    width: 8,
                    height: 8
                }
            )
            .is_ok()
        );
        finish(&mut p);
        assert_eq!(p.past.len(), 2);
        undo(&mut p);
        assert_eq!(p.image.pixels, original);
        redo(&mut p);
        assert_eq!(at(&p, 6, 6), [255, 0, 0, 255]);
        assert!(
            delete_region(
                &mut p,
                Region {
                    x: 6,
                    y: 6,
                    width: 8,
                    height: 8
                }
            )
            .is_ok()
        );
        assert_eq!(at(&p, 6, 6), [255; 4]);
        undo(&mut p);
        assert_eq!(at(&p, 6, 6), [255, 0, 0, 255]);
        assert!(
            transform_start(
                &mut p,
                Region {
                    x: u32::MAX,
                    y: 0,
                    width: 4,
                    height: 4
                }
            )
            .is_err()
        );
        assert!(
            transform_start(
                &mut p,
                Region {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 4
                }
            )
            .is_err()
        );
    }

    #[test]
    fn bounded_history_and_shape_previews() {
        let mut p = create(16, 16).ok().unwrap();
        for n in 0..80 {
            begin(&mut p, 0., 0., Tool::Fill, n, 1);
            finish(&mut p);
        }
        assert!(p.past.len() <= HISTORY_STEPS);
        begin(&mut p, 2., 2., Tool::Line, 0xff0000, 1);
        move_to(&mut p, 12., 2.);
        move_to(&mut p, 2., 12.);
        finish(&mut p);
        assert_ne!(at(&p, 12, 2), [255, 0, 0, 255]);
        assert_eq!(at(&p, 2, 8), [255, 0, 0, 255]);
        let mut diagonal = create(512, 512).ok().unwrap();
        begin(&mut diagonal, 0., 0., Tool::Line, 0xff0000, 1);
        move_to(&mut diagonal, 511., 511.);
        finish(&mut diagonal);
        for n in 0..512 {
            assert_eq!(at(&diagonal, n, n), [255, 0, 0, 255]);
        }
        assert_eq!(at(&diagonal, 256, 270), [255; 4]);
        let mut large = create(512, 512).ok().unwrap();
        for color in 0..30 {
            begin(&mut large, 0., 0., Tool::Brush, color, 1);
            finish(&mut large);
        }
        assert!(large.past.iter().map(|s| s.data.len()).sum::<usize>() <= HISTORY_BYTES);
        let before = p.image.pixels.clone();
        begin(&mut p, f64::NAN, 0., Tool::Brush, 0, 64);
        finish(&mut p);
        assert_eq!(before, p.image.pixels);
    }

#[test]
fn undo_redo_move_the_existing_pixel_buffers() {
    let mut paint = create(8, 8).ok().unwrap();
    begin(&mut paint, 1., 1., Tool::Brush, 0xff0000, 1);
    finish(&mut paint);
    let drawn = paint.image.pixels.as_ptr();
    let before = paint.past.back().unwrap().data.as_ptr();
    undo(&mut paint);
    assert_eq!(paint.image.pixels.as_ptr(), before);
    assert_eq!(paint.future.last().unwrap().data.as_ptr(), drawn);
    redo(&mut paint);
    assert_eq!(paint.image.pixels.as_ptr(), drawn);
    assert_eq!(paint.past.back().unwrap().data.as_ptr(), before);
}
