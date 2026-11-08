use super::{IndexBuilder, IndexError, corpus, query};

#[test]
fn chunk_boundaries_escaped_text_and_unterminated_tail() {
    let mut builder = IndexBuilder::default();
    builder.push(r#"{"text":"hello "#).unwrap();
    assert!(!builder.batch().unwrap());
    builder
        .push(concat!(
            "λ",
            r#"","other":4}"#,
            "\n\n",
            r#"{"text":"escaped \n value"}"#
        ))
        .unwrap();
    while builder.batch().unwrap() {}
    let mut owner = corpus();
    let index = builder.commit(&mut owner).unwrap();
    assert_eq!(query(&index, ""), ["hello λ", "escaped \n value"]);
}

#[test]
fn malformed_records_do_not_publish_partial_indices() {
    for line in [
        "{}",
        r#"["hello"]"#,
        "[]",
        r#"{"text":null}"#,
        r#"{"text":3}"#,
        r#"{"text":"a","text":"b"}"#,
    ] {
        let mut builder = IndexBuilder::default();
        builder.push(line).unwrap();
        let mut owner = corpus();
        assert!(matches!(
            builder.commit(&mut owner),
            Err(IndexError::InvalidRecord)
        ));
        assert!(owner.index.is_empty());
        assert_eq!(owner.revision, 0);
    }
}
