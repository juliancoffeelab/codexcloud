    #[test]
    fn accepts_public_api_shape_and_rejects_invalid_images() {
        assert_eq!(
            super::parse_cat(r#"[{"id":"λ","url":"https://cdn.example/cat.jpg"}]"#)
                .unwrap()
                .id,
            "λ"
        );
        for body in [
            "[]",
            "not JSON",
            r#"[{"id":"bad","url":"javascript:alert(1)"}]"#,
        ] {
            assert!(matches!(
                super::parse_cat(body),
                Err(super::CatError::InvalidResponse)
            ));
        }
    }
