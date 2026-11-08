#[ocamljsify::ocamljsify]
#[derive(Debug, PartialEq)]
/// Expected calculation failures.
pub enum CalculationError {
    /// Division requires a nonzero divisor.
    DivisionByZero,
    InvalidOperation {
        operation: String,
    },
}
#[ocamljsify::ocamljsify]
/// A calculated value and its history.
pub struct Calculation {
    /// The numeric result.
    pub value: f64,
    pub history: Vec<String>,
}
#[ocamljs_bindgen]
/// Evaluates an operation and returns its history.
pub fn calculate(a: f64, b: f64, operation: String) -> Result<Calculation, CalculationError> {
    let value = match operation.as_str() {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" if b == 0.0 => return Err(CalculationError::DivisionByZero),
        "/" => a / b,
        _ => return Err(CalculationError::InvalidOperation { operation }),
    };
    Ok(Calculation {
        value,
        history: vec![format!("{a} {operation} {b} = {value}")],
    })
}


#[ocamljs_bindgen(throws)]
/// Divides two numbers, raising on division by zero.
pub fn divide(a: f64, b: f64) -> Result<f64, CalculationError> {
    if b == 0.0 {
        Err(CalculationError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}
#[ocamljs_bindgen(opaque)]
/// A counter stored in Rust.
pub struct Counter {
    value: u32,
}
#[ocamljs_bindgen(module = Counter, name = "create")]
pub fn counter(value: u32) -> Counter {
    Counter { value }
}
#[ocamljs_bindgen(module = Counter)]
pub fn increment(counter: &mut Counter, amount: u32) -> u32 {
    counter.value = counter.value.wrapping_add(amount);
    counter.value
}
#[ocamljs_bindgen(module = Counter)]
pub fn current(counter: &Counter) -> u32 {
    counter.value
}
#[ocamljs_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}
#[ocamljs_bindgen]
pub fn tuple_distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}
#[ocamljs_bindgen]
pub fn tuple_bits(value: (u64, bool)) -> u64 {
    if value.1 { value.0 } else { 0 }
}
#[ocamljs_bindgen]
pub fn nested_tuple(value: ((i32, u8), f64)) -> f64 {
    f64::from(value.0.0) + f64::from(value.0.1) + value.1
}
#[ocamljs_bindgen]
pub fn echo_text(text: String) -> String {
    text
}
#[ocamljs_bindgen]
pub fn nested(value: Option<Option<String>>) -> Option<Option<String>> {
    value
}
#[ocamljs_bindgen]
pub fn bytes_sum(bytes: &[u8]) -> u32 {
    bytes.iter().map(|b| *b as u32).sum()
}
#[ocamljs_bindgen]
pub fn string_length(value: &str) -> u32 {
    value.chars().count() as u32
}
#[ocamljs_bindgen(module = Counter)]
pub fn same(a: &Counter, b: &mut Counter) -> u32 {
    b.value = a.value;
    b.value
}
#[ocamljs_bindgen]
pub fn int64(value: i64) -> i64 {
    value
}
#[ocamljs_bindgen]
pub fn unicode() -> String {
    "λ🙂\0".to_owned()
}
#[ocamljs_bindgen]
pub fn high_unsigned() -> u32 {
    u32::MAX
}
#[ocamljs_bindgen]
pub fn uint64(value: u64) -> u64 {
    value
}
#[ocamljsify::ocamljsify]
pub struct Wrapper<Item> {
    pub value: Item,
}
#[ocamljs_bindgen]
pub fn generic_roundtrip(a: Wrapper<u8>, b: Wrapper<String>) -> (Wrapper<String>, Wrapper<u8>) {
    (b, a)
}
#[ocamljsify::ocamljsify]
pub struct Empty();
#[ocamljs_bindgen]
pub fn empty(value: Empty) -> Empty {
    value
}
#[ocamljs_bindgen(opaque)]
pub struct HTTPServer {
    callback: Box<dyn Fn() -> u32>,
}
#[ocamljs_bindgen(module = HTTPServer, name = "create")]
pub fn http_server() -> HTTPServer {
    HTTPServer {
        callback: Box::new(|| 42),
    }
}
#[ocamljs_bindgen(module = HTTPServer, name = "value")]
pub fn http_value(server: &HTTPServer) -> u32 {
    (server.callback)()
}
// Exercise mutually referring generated modules with real owned values.
#[ocamljs_bindgen(module = Counter, name = "server")]
pub fn server_for(counter: &Counter) -> HTTPServer {
    let value = counter.value;
    HTTPServer {
        callback: Box::new(move || value),
    }
}
#[ocamljs_bindgen(module = HTTPServer, name = "counter")]
pub fn counter_for(server: &HTTPServer) -> Counter {
    Counter {
        value: (server.callback)(),
    }
}
#[ocamljs_bindgen]
pub fn qualified(value: std::primitive::u32) -> std::primitive::u32 {
    value
}
#[ocamljs_bindgen]
pub fn qualified_bool(value: std::primitive::bool) -> std::primitive::bool {
    value
}
#[ocamljs_bindgen]
pub fn writer(instance: String, writer: String) -> String {
    format!("{instance}:{writer}")
}
#[ocamljs_bindgen(module = Counter)]
pub fn many_counters(count: u32) -> Vec<Counter> {
    (0..count.min(100_000))
        .map(|value| Counter { value })
        .collect()
}
#[ocamljs_bindgen(throws, module = Counter)]
pub fn throw_owned() -> Result<(), Counter> {
    Err(Counter { value: 42 })
}
#[ocamljs_bindgen(module = Counter)]
pub fn invalid_output() -> Vec<Counter> {
    (0..1_000_001).map(|value| Counter { value }).collect()
}
#[ocamljs_bindgen(module = Counter)]
pub fn oversize_mixed() -> (Counter, Vec<()>) {
    (Counter { value: 5 }, vec![(); 1_000_001])
}
#[ocamljs_bindgen]
pub fn scratch_name(__bytes: u32, text: &str) -> u32 {
    __bytes + text.chars().count() as u32
}
#[ocamljsify::ocamljsify]
pub struct CounterRecord {
    pub label: String,
    pub counter: Counter,
}
#[ocamljs_bindgen(module = Counter)]
pub fn counter_record() -> CounterRecord {
    CounterRecord {
        label: "opaque in native record".into(),
        counter: Counter { value: 17 },
    }
}
#[ocamljs_bindgen(opaque)]
pub struct Vec2D {
    v: u32,
}
#[ocamljs_bindgen(module = Vec2D, name = "create")]
pub fn vector() -> Vec2D {
    Vec2D { v: 2 }
}
#[ocamljs_bindgen(module = Vec2D)]
pub fn dimension(v: &Vec2D) -> u32 {
    v.v
}

#[ocamljsify::ocamljsify]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CollectionKey {
    pub label: String,
    pub rank: u64,
}
#[ocamljsify::ocamljsify]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MixedKey {
    Payload(u32),
    Empty,
}
#[ocamljsify::ocamljsify]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitKey {
    Zed,
    Alpha,
    Middle,
}
#[ocamljs_bindgen]
pub fn unit_key_order() -> std::collections::BTreeMap<UnitKey, u32> {
    [(UnitKey::Middle, 3), (UnitKey::Zed, 1), (UnitKey::Alpha, 2)].into()
}
#[ocamljsify::ocamljsify]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ExplicitKey {
    Z = 1,
    A = -1,
}
#[ocamljsify::ocamljsify(key = "structural")]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ExplicitContractKey {
    Z = 1,
    A = -1,
}
#[ocamljsify::ocamljsify]
pub struct Collections {
    pub words: std::collections::BTreeMap<String, Vec<u32>>,
    pub counts: std::collections::HashMap<String, u32>,
    pub optional: std::collections::BTreeMap<String, Option<u32>>,
    pub vectors: std::collections::BTreeMap<Vec<u32>, String>,
    pub unsigned: std::collections::BTreeMap<u64, String>,
    pub points: std::collections::BTreeMap<CollectionKey, String>,
    pub mixed: std::collections::BTreeMap<MixedKey, u32>,
}
#[ocamljs_bindgen]
pub fn collections(value: Collections) -> Collections {
    value
}
#[ocamljs_bindgen]
pub fn sample_collections() -> Collections {
    Collections {
        words: [
            ("__proto__".into(), vec![1]),
            ("constructor".into(), vec![2]),
            ("\u{e000}".into(), vec![3]),
            ("\u{10000}".into(), vec![4]),
        ]
        .into(),
        counts: [("__proto__".into(), 11), ("constructor".into(), 12)].into(),
        optional: [("none".into(), None), ("some".into(), Some(7))].into(),
        vectors: [
            (vec![0, 2], "first".into()),
            (vec![1], "second".into()),
            (vec![1, 0], "third".into()),
        ]
        .into(),
        unsigned: [(0, "zero".into()), (u64::MAX, "max".into())].into(),
        points: [
            (
                CollectionKey {
                    label: "same".into(),
                    rank: 0,
                },
                "zero".into(),
            ),
            (
                CollectionKey {
                    label: "same".into(),
                    rank: u64::MAX,
                },
                "max".into(),
            ),
        ]
        .into(),
        mixed: [(MixedKey::Payload(9), 1), (MixedKey::Empty, 2)].into(),
    }
}
#[ocamljs_bindgen(module = Counter)]
pub fn owned_map() -> std::collections::BTreeMap<String, Counter> {
    [
        ("first".into(), Counter { value: 10 }),
        ("second".into(), Counter { value: 20 }),
    ]
    .into()
}

#[ocamljs_bindgen]
pub fn many_numbers(count: u32) -> Vec<u32> {
    (0..count.min(20_000)).collect()
}

#[ocamljs_bindgen]
pub fn fixed_numbers() -> [u32; 3] {
    [3, 2, 1]
}
static MAP_DROPS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
#[ocamljs_bindgen(opaque)]
pub struct MapOwned;
impl Drop for MapOwned {
    fn drop(&mut self) {
        MAP_DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
#[ocamljs_bindgen(module = MapOwned, name = "create")]
pub fn map_owner() -> MapOwned {
    MapOwned
}
#[ocamljs_bindgen(module = MapOwned)]
pub fn map_owners() -> std::collections::BTreeMap<String, MapOwned> {
    [("first".into(), MapOwned), ("second".into(), MapOwned)].into()
}
#[ocamljs_bindgen(module = MapOwned)]
pub fn map_drop_count() -> u32 {
    MAP_DROPS.load(std::sync::atomic::Ordering::SeqCst)
}

// Ordinary modules have no selected type or constructor. Both modules may use
// the same Rust type names without leaking their private carrier names.
#[ocamljs_bindgen]
pub mod geometry {

    #[ocamljsify::ocamljsify]
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }
    #[ocamljsify::ocamljsify]
    pub struct Bounds {
        pub min: Point,
        pub max: Point,
    }
    #[ocamljsify::ocamljsify]
    pub enum Axis {
        X,
        Y,
    }
    #[ocamljsify::ocamljsify]
    pub type Points = Vec<Point>;
    #[ocamljs_bindgen]
    pub fn distance(a: Point, b: Point) -> f64 {
        (a.x - b.x).hypot(a.y - b.y)
    }
    #[ocamljs_bindgen]
    pub fn contains(bounds: Bounds, point: Point) -> bool {
        (bounds.min.x..=bounds.max.x).contains(&point.x)
            && (bounds.min.y..=bounds.max.y).contains(&point.y)
    }
    #[ocamljs_bindgen]
    pub fn project(point: Point, axis: Axis) -> f64 {
        match axis {
            Axis::X => point.x,
            Axis::Y => point.y,
        }
    }
    #[ocamljs_bindgen]
    pub fn integer(point: Point) -> super::integer_geometry::Point {
        super::integer_geometry::Point {
            x: point.x as u8,
            y: point.y as u8,
        }
    }
    #[ocamljs_bindgen]
    pub fn origin() -> Point {
        Point { x: 0.0, y: 0.0 }
    }
    #[ocamljs_bindgen]
    pub fn echo(points: Points) -> Points {
        points
    }
    #[ocamljs_bindgen(throws)]
    pub fn checked_distance(a: Point, b: Point) -> Result<f64, Axis> {
        let distance = distance(a, b);
        if distance.is_finite() {
            Ok(distance)
        } else {
            Err(Axis::X)
        }
    }
}

#[ocamljs_bindgen(t = Point)]
pub mod integer_geometry {
    #[ocamljsify::ocamljsify]
    pub struct Point {
        pub x: u8,
        pub y: u8,
    }
    #[ocamljsify::ocamljsify]
    pub enum Axis {
        X,
        Y,
    }
    #[ocamljs_bindgen]
    pub fn floating(point: Point) -> super::geometry::Point {
        super::geometry::Point {
            x: f64::from(point.x),
            y: f64::from(point.y),
        }
    }
    #[ocamljs_bindgen]
    pub fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
    #[ocamljs_bindgen]
    pub fn project(point: Point, axis: Axis) -> u8 {
        match axis {
            Axis::X => point.x,
            Axis::Y => point.y,
        }
    }
}

#[ocamljs_bindgen]
pub mod arithmetic {
    #[ocamljs_bindgen]
    pub fn add(a: f64, b: f64) -> f64 {
        a + b
    }
    // This is a normal user function, not generated resource disposal.
    #[ocamljs_bindgen]
    pub fn dispose() -> bool {
        true
    }
}

// Type-only and empty modules also remain real public namespaces.
#[ocamljs_bindgen]
pub mod labels {
    #[ocamljsify::ocamljsify]
    pub struct Label<Item> {
        pub value: Item,
    }
}
#[ocamljs_bindgen]
pub mod empty_namespace {}

#[ocamljs_bindgen]
pub fn geometry_identity(point: geometry::Point) -> geometry::Point {
    point
}

#[ocamljs_bindgen(t = Axis)]
pub mod direction {
    #[ocamljsify::ocamljsify]
    pub enum Axis {
        X,
        Y,
    }
    #[ocamljs_bindgen::ocamljs_bindgen]
    pub fn flip(axis: Axis) -> Axis {
        match axis {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        }
    }
}
#[ocamljs_bindgen]
pub fn label_identity(label: labels::Label<String>) -> labels::Label<String> {
    label
}

// No local producer and no constructor designation. Every handle is delivered
// by an unrelated factory module; Drop remains attached to the Rust type.
#[ocamljs_bindgen(opaque_t = Token)]
pub mod token {
    use std::cell::Cell;
    thread_local! { static DROPS: Cell<u32> = const { Cell::new(0) }; }
    pub struct Token {
        pub(crate) value: u32,
    }
    impl Drop for Token {
        fn drop(&mut self) {
            DROPS.set(DROPS.get() + 1);
        }
    }
    #[ocamljs_bindgen::ocamljs_bindgen]
    pub fn value(token: &Token) -> u32 {
        token.value
    }
    #[ocamljs_bindgen::ocamljs_bindgen]
    pub fn drops() -> u32 {
        DROPS.get()
    }
}
#[ocamljs_bindgen]
pub mod tokens {
    #[ocamljs_bindgen::ocamljs_bindgen]
    pub fn parse(text: String) -> Result<super::token::Token, String> {
        text.parse()
            .map(|value| super::token::Token { value })
            .map_err(|_| "invalid token".into())
    }
    #[ocamljs_bindgen::ocamljs_bindgen]
    pub fn optional(present: bool) -> Option<super::token::Token> {
        present.then(|| super::token::Token { value: 42 })
    }
    #[ocamljs_bindgen::ocamljs_bindgen]
    pub fn batch() -> Vec<super::token::Token> {
        vec![
            super::token::Token { value: 7 },
            super::token::Token { value: 9 },
        ]
    }
}

// Exercises fixed runs on both sides of an opaque claim, including pooled
// Unicode and a transparent newtype; the closed enum splits a run and reuses
// its compiler-native decoder.
#[ocamljsify::ocamljsify(transparent)]
pub struct BatchId(pub u32);
#[ocamljsify::ocamljsify]
pub struct BatchRecord {
    pub first: String,
    pub id: BatchId,
    pub flag: bool,
    pub key: Key,
    pub owner: Counter,
    pub second: String,
    pub tail: u16,
}
#[ocamljs_bindgen(module = Counter)]
pub fn batched_output() -> BatchRecord {
    BatchRecord {
        first: "\u{feff}λ".into(),
        id: BatchId(17), flag: true, key: Key::Equals,
        owner: Counter { value: 23 }, second: "🦀\0".into(), tail: 513,
    }
}

#[ocamljs_bindgen]
pub fn text_pair(first: String, second: String) -> (String, String) {
    (first, second)
}

// A real exported name guards optional equality-helper collision validation.
#[ocamljs_bindgen]
pub fn equal_empty() -> bool { true }

#[ocamljsify::ocamljsify]
pub struct NumericSamples {
    pub values: Vec<f64>,
    pub ordinals: Vec<u32>,
}
#[ocamljs_bindgen]
pub fn float_samples(empty: bool) -> NumericSamples {
    if empty {
        NumericSamples { values: Vec::new(), ordinals: Vec::new() }
    } else {
        NumericSamples {
            values: vec![1.25, -3.5, -0.0, 0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 2.75],
            ordinals: (0..20_000).collect(),
        }
    }
}

#[ocamljsify::ocamljsify]
pub type DepthAlias0<T> = DepthAlias1<T>;
#[ocamljsify::ocamljsify]
pub type DepthAlias1<T> = DepthAlias2<T>;
#[ocamljsify::ocamljsify]
pub type DepthAlias2<T> = DepthAlias3<T>;
#[ocamljsify::ocamljsify]
pub type DepthAlias3<T> = DepthAlias4<T>;
#[ocamljsify::ocamljsify]
pub type DepthAlias4<T> = DepthAlias5<T>;
#[ocamljsify::ocamljsify]
pub type DepthAlias5<T> = DepthAlias6<T>;
#[ocamljsify::ocamljsify]
pub type DepthAlias6<T> = DepthAlias7<T>;
#[ocamljsify::ocamljsify]
pub type DepthAlias7<T> = T;
#[ocamljsify::ocamljsify]
pub struct DepthProbeNode {
    pub children: DepthAlias0<Vec<DepthProbeNode>>,
}
#[ocamljs_bindgen]
pub fn depth_probe(depth: u32) -> DepthProbeNode {
    let mut node = DepthProbeNode { children: Vec::new() };
    for _ in 1..depth {
        node = DepthProbeNode { children: vec![node] };
    }
    node
}

// A single fixed DTO graph exercises opt-in JavaScript projection without
// changing the calculator's ordinary compiler-native API.
#[ocamljsify::ocamljsify(transparent, js_value)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ProjectionKey(pub String);
#[ocamljsify::ocamljsify(js_value)]
pub type ProjectionKeyAlias = ProjectionKey;
#[ocamljsify::ocamljsify(js_value)]
pub type ProjectionText = String;
#[ocamljsify::ocamljsify(js_value)]
pub type ProjectionWrappedText = std::sync::Arc<std::rc::Rc<Box<String>>>;
#[ocamljsify::ocamljsify(js_value)]
pub enum ProjectionCase {
    Empty,
    Flag(bool),
    Named { text: String },
    __PROTO__ { value: bool },
}
#[ocamljsify::ocamljsify(js_value)]
pub struct ProjectionSample {
    pub first: ProjectionText,
    pub second: String,
    pub third: String,
    pub flags: Vec<bool>,
    pub numbers: [f64; 4],
    pub signed: i64,
    pub unsigned: u64,
    pub options: Vec<Option<Option<String>>>,
    pub key: Key,
    pub cases: Vec<ProjectionCase>,
    pub pair: (bool, String),
    pub words: std::collections::BTreeMap<ProjectionKeyAlias, Vec<String>>,
    pub counts: std::collections::HashMap<String, u32>,
    pub wrapped: ProjectionWrappedText,
    pub wrapped_words: std::collections::BTreeMap<ProjectionWrappedText, String>,
    pub r#type: String,
    pub trailing_: String,
    pub under_score: String,
    pub double__under: String,
    pub nested: Result<String, bool>,
}
#[ocamljsify::ocamljsify]
pub type ProjectionResult = Result<ProjectionSample, CalculationError>;
#[ocamljs_bindgen]
pub fn projection_sample(fail: bool) -> ProjectionResult {
    if fail {
        return Err(CalculationError::InvalidOperation { operation: "native λ\0".into() });
    }
    Ok(ProjectionSample {
        first: "\u{feff}λ".into(),
        r#type: "keyword".into(),
        trailing_: "trailing".into(),
        under_score: "underscore".into(),
        double__under: "double".into(),
        nested: Ok("nested".into()),
        second: "🙂\0".into(),
        third: "\u{e000}🦀".into(),
        flags: vec![false, true],
        numbers: [-0.0, 1.25, f64::INFINITY, f64::NAN],
        signed: i64::MIN,
        unsigned: u64::MAX,
        options: vec![None, Some(None), Some(Some("λ".into()))],
        key: Key::Equals,
        cases: vec![ProjectionCase::Empty, ProjectionCase::Flag(true), ProjectionCase::Named { text: "payload 🦀".into() }, ProjectionCase::__PROTO__ { value: true }],
        pair: (true, "tuple\0".into()),
        words: ["", "__proto__", "constructor", "λ", "🙂", "\u{e000}"]
            .into_iter().map(|key| (ProjectionKey(key.into()), vec![format!("value:{key}")])).collect(),
        counts: [("__proto__".into(), 7), ("constructor".into(), 9)].into(),
        wrapped: std::sync::Arc::new(std::rc::Rc::new(Box::new("wrapped λ\0".into()))),
        wrapped_words: ["__proto__", "λ"].into_iter().map(|key| (
            std::sync::Arc::new(std::rc::Rc::new(Box::new(key.into()))),
            format!("wrapped:{key}"),
        )).collect(),
    })
}
#[ocamljs_bindgen(js_value)]
pub fn projection_value(fail: bool) -> Result<ProjectionSample, CalculationError> {
    projection_sample(fail)
}
#[ocamljs_bindgen(throws, js_value)]
pub fn projection_throw(fail: bool) -> Result<ProjectionSample, CalculationError> {
    projection_sample(fail)
}


#[ocamljs_bindgen(js_value, module = Counter, name = "sample")]
pub fn projection_owned(counter: &Counter, fail: bool) -> Result<ProjectionSample, CalculationError> {
    projection_sample(fail || counter.value == u32::MAX)
}
#[ocamljs_bindgen(throws, js_value, module = Counter, name = "checked")]
pub fn projection_owned_throw(counter: &Counter, fail: bool) -> Result<ProjectionSample, CalculationError> {
    projection_owned(counter, fail)
}


#[ocamljs_bindgen(js_value)]
pub fn projection_tuple() -> (bool, String) {
    (true, "tuple root λ".into())
}

#[ocamljsify::ocamljsify(js_value)]
pub struct ProjectionRecursive {
    pub label: String,
    pub children: Vec<ProjectionRecursive>,
}
#[ocamljs_bindgen(js_value)]
pub fn projection_recursive() -> ProjectionRecursive {
    ProjectionRecursive {
        label: "parent".into(),
        children: vec![ProjectionRecursive { label: "child".into(), children: vec![] }],
    }
}


#[ocamljsify::ocamljsify]
pub struct ProjectionFlatError { pub code: u32 }
#[ocamljs_bindgen(js_value)]
pub fn projection_flat_error(fail: bool) -> Result<ProjectionSample, ProjectionFlatError> {
    if fail { return Err(ProjectionFlatError { code: 13 }); }
    projection_sample(false).map_err(|_| ProjectionFlatError { code: 8 })
}


#[ocamljsify::ocamljsify]
pub struct ProjectionNativeItem { pub flag: bool, pub text: String }
#[ocamljsify::ocamljsify(js_value)]
pub struct ProjectionMixed {
    pub rows: Vec<ProjectionNativeItem>,
    pub item: ProjectionNativeItem,
}
#[ocamljs_bindgen]
pub fn projection_mixed() -> ProjectionMixed {
    ProjectionMixed { rows: vec![ProjectionNativeItem { flag: true, text: "native λ".into() }], item: ProjectionNativeItem { flag: false, text: "native item".into() } }
}
#[ocamljs_bindgen]
pub fn projection_mixed_echo(value: ProjectionMixed) -> ProjectionMixed { value }
#[ocamljsify::ocamljsify]
pub struct ProjectionEnvelope {
    #[ocamljsify(js_value)]
    pub rows: Vec<ProjectionNativeItem>,
    pub item: ProjectionNativeItem,
    pub result: ProjectionResult,
}
#[ocamljs_bindgen]
pub fn projection_envelope() -> ProjectionEnvelope {
    let mixed=projection_mixed();
    ProjectionEnvelope { rows: mixed.rows, item: mixed.item, result: projection_sample(false) }
}
#[ocamljs_bindgen]
pub fn projection_envelope_echo(value: ProjectionEnvelope) -> ProjectionEnvelope { value }
#[ocamljs_bindgen]
pub fn projection_roundtrip(value: ProjectionSample) -> ProjectionSample { value }
#[ocamljsify::ocamljsify(js_value)]
pub type ProjectionJsResult = Result<ProjectionSample, CalculationError>;
#[ocamljs_bindgen(js_value)]
pub fn projection_js_result(fail: bool) -> ProjectionJsResult { projection_sample(fail) }
#[ocamljsify::ocamljsify]
pub struct ProjectionResultEnvelope { pub value: ProjectionJsResult }
#[ocamljs_bindgen]
pub fn projection_js_result_envelope(fail: bool) -> ProjectionResultEnvelope { ProjectionResultEnvelope { value: projection_sample(fail) } }

#[ocamljs_bindgen]
pub fn projection_recursive_echo(value: ProjectionRecursive) -> ProjectionRecursive { value }

// One representative matrix exercises explicit selectors and the ABI paths they affect.
#[ocamljsify::ocamljsify(js_value("object"))]
pub struct SelectorSample {
    #[ocamljsify(js_value("number"))]
    pub number: u64,
    #[ocamljsify(js_value("bigint"))]
    pub bigint: i16,
    #[ocamljsify(js_value("boolean"))]
    pub boolean: bool,
    #[ocamljsify(js_value("string"))]
    pub string: String,
    #[ocamljsify(js_value("array"))]
    pub array: std::collections::LinkedList<u16>,
    #[ocamljsify(js_value("typed_array"))]
    pub bytes: Vec<u8>,
    #[ocamljsify(js_value("typed_array"))]
    pub floats: [f64; 2],
    #[ocamljsify(js_value("typed_array"))]
    pub wide: Vec<u64>,
    #[ocamljsify(js_value("dict"))]
    pub dict: std::collections::BTreeMap<String, i32>,
    #[ocamljsify(js_value("map"))]
    pub map: std::collections::BTreeMap<i32, String>,
    #[ocamljsify(js_value("nullable"))]
    pub nullable: Option<Option<String>>,
    #[ocamljsify(js_value("optional"))]
    pub optional: Option<Option<String>>,
    #[ocamljsify(js_value("optional"))]
    pub unit: Option<()>,
}
#[ocamljs_bindgen]
pub fn selector_sample() -> SelectorSample {
    SelectorSample {
        number: 9007199254740991, bigint: i16::MIN, boolean: true,
        string: "\u{feff}λ🦀\0".into(), array: [0,65535].into_iter().collect(),
        bytes: vec![0,255], floats: [-0.0,f64::INFINITY], wide: vec![0,u64::MAX],
        dict: [("__proto__".into(),-1),("🙂".into(),2)].into_iter().collect(),
        map: [(-2,"negative".into()),(1,"positive".into())].into_iter().collect(),
        nullable: Some(None), optional: Some(Some("present".into())), unit: Some(()),
    }
}
#[ocamljs_bindgen]
pub fn selector_echo(value: SelectorSample) -> SelectorSample { value }
#[ocamljs_bindgen(js_value("number"))]
pub fn selector_number(value: u64) -> u64 { value }
#[ocamljs_bindgen(js_value("bigint"))]
pub fn selector_bigint(value: i16) -> i16 { value }
#[ocamljsify::ocamljsify(js_value("number"))]
pub type SelectorNumber = u64;
#[ocamljs_bindgen]
pub fn selector_alias(value: SelectorNumber) -> SelectorNumber { value }
#[ocamljs_bindgen]
pub async fn selector_alias_async(value: SelectorNumber) -> SelectorNumber { value }
#[ocamljs_bindgen(js_value("number"))]
pub async fn selector_scalar_async() -> u64 { 42 }
#[ocamljs_bindgen]
pub async fn selector_alias_error_async() -> Result<String, SelectorNumber> { Err(42) }

#[ocamljsify::ocamljsify(js_value)]
pub struct SelectorGeneric<T> { pub values: Vec<T> }
#[ocamljs_bindgen]
pub fn selector_generic(value: SelectorGeneric<ProjectionNativeItem>) -> SelectorGeneric<ProjectionNativeItem> { value }
#[ocamljsify::ocamljsify(js_value)]
pub struct SelectorFlat { pub x: f64, pub y: f64 }
#[ocamljs_bindgen]
pub fn selector_flat(value: SelectorFlat) -> SelectorFlat { value }
#[ocamljsify::ocamljsify]
pub struct SelectorMixedFlat {
    #[ocamljsify(js_value("number"))]
    pub x: i32,
    pub y: u32,
}
#[ocamljs_bindgen]
pub fn selector_mixed_flat(value: SelectorMixedFlat) -> SelectorMixedFlat { value }
#[ocamljs_bindgen(js_value)]
pub async fn selector_async() -> SelectorSample { selector_sample() }
#[ocamljs_bindgen]
pub fn selector_native_list(value: std::collections::LinkedList<i16>) -> std::collections::LinkedList<i16> { value }

// Generic aliases must select the same direct owner ABI as their concrete type.
#[ocamljs_bindgen(module = Counter, name = "aliased")]
pub fn aliased_counter(value: u32) -> DepthAlias0<Counter> { Counter { value } }

// A callback failure must dispose an unpublished direct owner, not an output ID.
#[ocamljsify::ocamljsify]
pub struct OwnerNotification { pub value: u32 }
#[ocamljs_bindgen(module = Counter)]
pub fn counter_after_callback(notify: ocamljs_bindgen::Callback<OwnerNotification>) -> Counter {
    let _ = notify.emit(OwnerNotification { value: 7 });
    Counter { value: 7 }
}

#[ocamljsify::ocamljsify(js_value("typed_array"))]
pub type SelectorFloatSlice = [f64];
#[ocamljs_bindgen]
pub fn selector_slice_sum(values: &SelectorFloatSlice) -> f64 {
    values.iter().sum()
}

#[ocamljsify::ocamljsify(js_value("string"))]
pub type SelectorText = String;
#[ocamljsify::ocamljsify(js_value("string"))]
pub type SelectorBorrowedText = str;
#[ocamljs_bindgen]
pub type NativeText = String;
#[ocamljs_bindgen]
pub type NativeBorrowedText = str;
#[ocamljs_bindgen]
pub fn selector_text_echo(value: SelectorText) -> SelectorText { value }
#[ocamljs_bindgen]
pub fn selector_text_borrowed(value: &SelectorBorrowedText) -> u32 { value.len() as u32 }
#[ocamljs_bindgen]
pub fn native_alias_echo(value: NativeText) -> NativeText { value }
#[ocamljs_bindgen]
pub fn native_alias_borrowed(value: &NativeBorrowedText) -> u32 { value.len() as u32 }
#[ocamljs_bindgen]
pub fn selector_text_notify(value: SelectorText, progress: ocamljs_bindgen::Callback<OwnerNotification>) -> SelectorText {
    let _ = progress.emit(OwnerNotification { value: value.len() as u32 });
    value
}

#[ocamljsify::ocamljsify(js_value)]
pub type SelectedTextResult = Result<String, u64>;
#[ocamljs_bindgen]
pub fn selected_text_result(fail: bool) -> SelectedTextResult {
    if fail { Err(u64::MAX) } else { Ok("selected λ".into()) }
}
#[ocamljs_bindgen]
pub async fn selected_text_result_async(fail: bool) -> SelectedTextResult {
    selected_text_result(fail)
}
#[ocamljsify::ocamljsify(js_value)]
pub type SelectedUnitTextResult = Result<String, ()>;
#[ocamljs_bindgen]
pub async fn selected_unit_text_result_async() -> SelectedUnitTextResult { Err(()) }
