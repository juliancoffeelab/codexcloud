use ocamljs_bindgen::*;
struct Number(u32);
ocamljs_bindgen::opaque_storage!(Number);
struct Other;
ocamljs_bindgen::opaque_storage!(Other);
struct Reentrant;
ocamljs_bindgen::opaque_storage!(Reentrant);
use std::{cell::Cell, rc::Rc};
#[test]
fn codec_rejects_malformed_input_and_bounds_allocations() {
    assert!(decode::<bool>(&[2]).is_err());
    assert!(decode::<Option<u32>>(&[7]).is_err());
    assert!(decode::<u32>(&[1, 2, 3]).is_err());
    assert!(decode::<u8>(&[0, 1]).is_err());
    assert!(decode::<String>(&[1, 0, 0, 0, 255]).is_err());
    assert!(decode::<Vec<()>>(&1_000_001u32.to_le_bytes()).is_err());
    let mut d = Decoder::new(&[]).unwrap();
    d.items(MAX_ITEMS).unwrap();
    assert!(<[(); 1]>::decode(&mut d).is_err());
}
#[test]
fn opaque_aliases_types_and_stale_generations_are_checked() {
    unsafe {
        let id = insert(Number(7));
        assert_eq!(with_ref::<Number, _>(id, |x| x.0).unwrap(), 7);
        assert!(with_ref::<Other, _>(id, |_| ()).is_err());
        let conflict = with_ref::<Number, _>(id, |_| with_mut::<Number, _>(id, |_| ())).unwrap();
        assert!(conflict.is_err());
        with_mut::<Number, _>(id, |x| x.0 = 9).unwrap();
        dispose(id).unwrap();
        let replacement = insert(Number(12));
        assert_ne!(id, replacement);
        assert!(with_ref::<Number, _>(id, |_| ()).is_err());
        dispose(replacement).unwrap();
    }
}
struct Tracked(Rc<Cell<u32>>);
ocamljs_bindgen::opaque_storage!(Tracked);
impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
        let inner = insert(Reentrant);
        unsafe {
            dispose(inner).unwrap();
        }
    }
}
struct Partial(Tracked);
impl Encode for Partial {
    type Return = ocamljs_bindgen::abi::Buffer<Self>;
    fn encode(self, e: &mut Encoder) {
        "temporary pooled string".to_owned().encode(e);
        e.opaque(self.0);
        e.fail("output fixture failure");
    }
}
#[test]
fn failed_output_rolls_back_owned_handles_and_drops_outside_arena() {
    let baseline = __ocamljs_live();
    let drops = Rc::new(Cell::new(0));
    assert!(publish(Partial(Tracked(drops.clone()))).is_err());
    assert_eq!(drops.get(), 1);
    assert_eq!(__ocamljs_live(), baseline);
}
#[ocamljs_bindgen]
struct Node {
    next: Option<Box<Node>>,
}
#[test]
fn recursive_output_obeys_selected_policy() {
    let mut node = Node { next: None };
    for _ in 0..512 {
        node = Node {
            next: Some(Box::new(node)),
        };
    }
    let result = publish(node);
    if PARANOID {
        assert!(result.is_err());
    } else {
        __ocamljs_output_free(result.unwrap());
    }
}
#[test]
fn unsigned_and_nested_options_preserve_bits_and_tags() {
    for value in [None, Some(None), Some(Some(u64::MAX))] {
        let mut e = Encoder::new();
        value.encode(&mut e);
        assert_eq!(decode::<Option<Option<u64>>>(&e.bytes).unwrap(), value);
    }
}

#[ocamljs_bindgen(borrow)]
#[derive(Debug, PartialEq, EncodeProjection)]
struct SharedRecord {
    text: String,
    values: Vec<Option<u32>>,
}
struct SharedProjection<'a>(&'a SharedRecord, &'a Cell<usize>);
impl EncodeAs<SharedRecord> for SharedProjection<'_> {
    fn encode_as(self, out: &mut Encoder) {
        self.0.encode_ref(out);
        self.1.set(out.bytes.as_ptr() as usize);
    }
}
#[test]
fn borrowed_projection_keeps_wire_shape_and_transfers_the_return_buffer() {
    let value = SharedRecord {
        text: "borrowed".into(),
        values: vec![Some(7), None],
    };
    let projection = SharedRecordProjection {
        text: &value.text,
        values: &value.values,
    };
    let mut projected = Encoder::new();
    projection.encode_as(&mut projected);
    let mut expected = Encoder::new();
    value.encode_ref(&mut expected);
    assert_eq!(projected.bytes, expected.bytes);
    let pointer = Cell::new(0);
    let encoded = Encoded::<SharedRecord>::new(SharedProjection(&value, &pointer));
    let output = abi::lower_output(encoded).unwrap();
    assert_eq!(__ocamljs_output_ptr(output), pointer.get());
    let bytes = unsafe {
        std::slice::from_raw_parts(
            __ocamljs_output_ptr(output) as *const u8,
            __ocamljs_output_len(output),
        )
    };
    let pool = unsafe {
        std::slice::from_raw_parts(
            __ocamljs_output_pool_ptr(output) as *const u8,
            __ocamljs_output_pool_len(output),
        )
    };
    assert_eq!(decode_segments::<SharedRecord>(bytes, pool).unwrap(), value);
    __ocamljs_output_free(output);
    // Sharing and decoding Arc retain precisely the pointee's data shape.
    let shared = std::sync::Arc::new(value);
    let mut owned = Encoder::new();
    shared.clone().encode(&mut owned);
    let mut borrowed = Encoder::new();
    shared.encode_ref(&mut borrowed);
    assert_eq!(owned.bytes, borrowed.bytes);
    assert_eq!(
        *decode::<std::sync::Arc<SharedRecord>>(&borrowed.bytes).unwrap(),
        *shared
    );
    assert_eq!(std::sync::Arc::strong_count(&shared), 1);
}

struct FailedProjection;
impl EncodeAs<SharedRecord> for FailedProjection {
    fn encode_as(self, out: &mut Encoder) {
        out.fail("projection fixture failure");
    }
}
#[test]
fn eager_projection_preserves_encoder_errors_and_rejects_nested_use() {
    let baseline = __ocamljs_live();
    assert!(abi::lower_output(Encoded::<SharedRecord>::new(FailedProjection)).is_err());
    let value = SharedRecord {
        text: "detached".into(),
        values: vec![],
    };
    let pointer = Cell::new(0);
    assert!(
        publish(Some(Encoded::<SharedRecord>::new(SharedProjection(
            &value, &pointer
        ))))
        .is_err()
    );
    assert_eq!(__ocamljs_live(), baseline);
}

#[ocamljs_bindgen(opaque)]
struct NestedOwnedHandle(u32);
#[ocamljs_bindgen]
struct OwnedHandleRecord {
    handle: NestedOwnedHandle,
}
#[ocamljs_bindgen]
struct OwnedHostRecord {
    text: ocamljs_bindgen::host::JsString,
}
#[test]
fn default_structural_codec_still_transfers_owned_opaque_handles() {
    unsafe {
        let output = publish(OwnedHandleRecord {
            handle: NestedOwnedHandle(42),
        })
        .unwrap();
        let bytes = std::slice::from_raw_parts(
            __ocamljs_output_ptr(output) as *const u8,
            __ocamljs_output_len(output),
        );
        let handle = decode::<u32>(bytes).unwrap();
        assert_eq!(
            with_ref::<NestedOwnedHandle, _>(handle, |value| value.0).unwrap(),
            42
        );
        __ocamljs_output_free(output);
        assert!(with_ref::<NestedOwnedHandle, _>(handle, |_| ()).is_err());
    }
}

#[ocamljs_bindgen]
#[derive(EncodeProjection)]
enum ProjectedList {
    Empty,
    Values { values: Vec<u32> },
}
#[test]
fn enum_and_exact_size_sequence_projections_derive_the_owned_layout() {
    let values = vec![4, 7, 9];
    let projection = ProjectedListProjection::Values {
        values: SequenceProjection(values.iter()),
    };
    let mut actual = Encoder::new();
    projection.encode_as(&mut actual);
    assert!(
        matches!(decode::<ProjectedList>(&actual.bytes).unwrap(),ProjectedList::Values{values:v} if v==values)
    );
    let projection: ProjectedListProjection<SequenceProjection<std::slice::Iter<'_, u32>>> =
        ProjectedListProjection::Empty;
    let mut actual = Encoder::new();
    projection.encode_as(&mut actual);
    assert!(matches!(
        decode::<ProjectedList>(&actual.bytes).unwrap(),
        ProjectedList::Empty
    ));
}

#[test]
fn result_projection_preserves_the_error_envelope() {
    let result: Result<SharedRecordProjection<&String, &Vec<Option<u32>>>, u32> = Err(17);
    let mut actual = Encoder::new();
    <ResultProjection<_, _> as EncodeAs<Result<SharedRecord, u32>>>::encode_as(
        ResultProjection(result),
        &mut actual,
    );
    let mut expected = Encoder::new();
    Result::<SharedRecord, u32>::Err(17).encode(&mut expected);
    assert_eq!(actual.bytes, expected.bytes);
}

#[test]
fn segmented_strings_preserve_inline_codec_and_move_owned_pool_storage() {
    let value = "\u{feff}λ🦀\0".to_owned();
    let pointer = value.as_ptr() as usize;
    let output = publish(value.clone()).unwrap();
    let mut inline = Encoder::default();
    assert!(!inline.is_publication());
    value.clone().encode(&mut inline);
    assert_eq!(decode::<String>(&inline.bytes).unwrap(), value);
    __ocamljs_output_free(output);
    let output = publish(value).unwrap();
    assert_eq!(__ocamljs_output_pool_ptr(output), pointer);
    assert_eq!(__ocamljs_output_len(output), 4);
    __ocamljs_output_free(output);

    let mut oversized = String::with_capacity(1024);
    oversized.push('x');
    let oversized_pointer = oversized.as_ptr() as usize;
    let output = publish(oversized).unwrap();
    assert_ne!(__ocamljs_output_pool_ptr(output), oversized_pointer);
    assert_eq!(__ocamljs_output_pool_len(output), 1);
    __ocamljs_output_free(output);

    let values = vec!["".to_owned(), "é".into(), "\u{feff}".into(), "🦀\0".into()];
    let output = publish(values.clone()).unwrap();
    let (body, pool) = unsafe {
        (
            std::slice::from_raw_parts(
                __ocamljs_output_ptr(output) as *const u8,
                __ocamljs_output_len(output),
            ),
            std::slice::from_raw_parts(
                __ocamljs_output_pool_ptr(output) as *const u8,
                __ocamljs_output_pool_len(output),
            ),
        )
    };
    assert_eq!(body.len(), 4 + 4 * values.len());
    assert_eq!(pool, values.concat().as_bytes());
    assert_eq!(decode_segments::<Vec<String>>(body, pool).unwrap(), values);
    __ocamljs_output_free(output);
}
#[test]
fn segmented_decode_rejects_cross_string_utf8_and_trailing_pool_data() {
    // The concatenation is valid UTF-8, but each declared string is invalid.
    let mut lengths = Encoder::new();
    (1u32, 1u32).encode(&mut lengths);
    assert!(decode_segments::<(String, String)>(&lengths.bytes, "é".as_bytes()).is_err());
    assert!(decode_segments::<(String, String)>(&lengths.bytes, &[0xff, 0xff]).is_err());
    let mut empty = Encoder::new();
    0u32.encode(&mut empty);
    assert!(decode_segments::<String>(&empty.bytes, b"trailing").is_err());
}

#[test]
fn output_info_is_a_borrowed_four_word_descriptor() {
    let output = publish("descriptor λ".to_owned()).unwrap();
    // Capture immediately: calling another ABI helper may move the output slab.
    let words = unsafe { *(__ocamljs_output_info(output) as *const [usize; 4]) };
    assert_eq!(
        words,
        [
            __ocamljs_output_ptr(output),
            __ocamljs_output_len(output),
            __ocamljs_output_pool_ptr(output),
            __ocamljs_output_pool_len(output)
        ]
    );
    __ocamljs_output_free(output);
}

#[test]
fn bulk_numeric_transport_preserves_wire_and_float_bits() {
    let mut e = Encoder::new();
    vec![0x1234u16, 0xabcd].encode(&mut e);
    assert_eq!(e.bytes, vec![2, 0, 0, 0, 0x34, 0x12, 0xcd, 0xab]);
    assert_eq!(decode::<Vec<u16>>(&e.bytes).unwrap(), vec![0x1234, 0xabcd]);
    let values = vec![-0.0f64, f64::INFINITY, f64::from_bits(0x7ff8000000000123)];
    let mut e = Encoder::new();
    values.clone().encode(&mut e);
    let restored = decode::<Vec<f64>>(&e.bytes).unwrap();
    assert_eq!(
        values.iter().map(|n| n.to_bits()).collect::<Vec<_>>(),
        restored.iter().map(|n| n.to_bits()).collect::<Vec<_>>()
    );
    let mut e = Encoder::new();
    [1u32, 2, 3].encode(&mut e);
    assert_eq!(decode::<[u32; 3]>(&e.bytes).unwrap(), [1, 2, 3]);
}
