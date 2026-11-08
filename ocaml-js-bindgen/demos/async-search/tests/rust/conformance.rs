use std::{collections::VecDeque, pin::Pin, task::Context};

#[derive(Default)]
struct Channel {
    chunks: VecDeque<String>,
    closed: bool,
    waker: Option<Waker>,
}
#[ocamljs_bindgen(opaque)]
pub struct Input {
    channel: Rc<RefCell<Channel>>,
}
struct Next(Rc<RefCell<Channel>>);
impl Future for Next {
    type Output = Option<String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut c = self.0.borrow_mut();
        if let Some(chunk) = c.chunks.pop_front() {
            Poll::Ready(Some(chunk))
        } else if c.closed {
            Poll::Ready(None)
        } else {
            c.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[ocamljs_bindgen(module = Input, name = "create")]
pub fn input() -> Input {
    Input {
        channel: Rc::new(RefCell::new(Channel::default())),
    }
}
#[ocamljs_bindgen(module = Input)]
pub fn push(input: &Input, chunk: String) -> Result<(), IndexError> {
    if chunk.len() > 32768 {
        return Err(IndexError::TooLarge);
    }
    let wake = {
        let mut c = input.channel.borrow_mut();
        if c.closed {
            return Err(IndexError::Closed);
        }
        if c.chunks.len() >= 4 {
            return Err(IndexError::Full);
        }
        c.chunks.push_back(chunk);
        c.waker.take()
    };
    if let Some(w) = wake {
        w.wake();
    }
    Ok(())
}
#[ocamljs_bindgen(module = Input)]
pub fn finish(input: &Input) {
    let wake = {
        let mut c = input.channel.borrow_mut();
        c.closed = true;
        c.waker.take()
    };
    if let Some(w) = wake {
        w.wake();
    }
}
#[ocamljs_bindgen(module = Corpus)]
pub fn revision(corpus: &Corpus) -> u32 {
    corpus.revision
}
#[ocamljs_bindgen(module = Input)]
pub fn waiting(input: &Input) -> bool {
    input.channel.borrow().waker.is_some()
}

#[ocamljs_bindgen]
pub async fn digest(text: String) -> Result<u32, IndexError> {
    yield_now().await;
    Ok(text.chars().count() as u32)
}
#[ocamljs_bindgen(throws)]
pub async fn reject() -> Result<u32, IndexError> {
    yield_now().await;
    Err(IndexError::InvalidRecord)
}
#[ocamljs_bindgen]
pub async fn borrowed_text(text: &str, bytes: &[u8]) -> u32 {
    yield_now().await;
    text.chars().count() as u32 + bytes.iter().map(|n| *n as u32).sum::<u32>()
}
#[ocamljs_bindgen(module = Corpus)]
pub async fn shared(corpus: &Corpus) -> u32 {
    yield_now().await;
    corpus.revision
}

#[ocamljs_bindgen(module = Index, name = "create")]
pub async fn build_index(
    corpus: &mut Corpus,
    input: &Input,
    progress: Callback<Progress>,
) -> Result<Index, IndexError> {
    let mut builder = IndexBuilder::default();
    while let Some(chunk) = Next(input.channel.clone()).await {
        builder.push(&chunk)?;
        while builder.batch()? {
            builder.progress(&progress);
            yield_now().await;
        }
    }
    builder.commit(corpus)
}

#[ocamljs_bindgen]
pub fn force_trap() {
    panic!("intentional poison fixture");
}


#[ocamljs_bindgen(module = Index)]
pub fn build_sync(corpus: &mut Corpus, text: &str) -> Result<Index, IndexError> {
    if text.len() > 1024 * 1024 {
        return Err(IndexError::TooLarge);
    }
    let mut builder = IndexBuilder::default();
    for line in text.lines() {
        IndexBuilder::line(&mut builder.entries, line)?;
    }
    builder.commit(corpus)
}
struct NotifyDrop(Callback<Progress>);
impl Drop for NotifyDrop {
    fn drop(&mut self) {
        let _ = self.0.emit(Progress { records: 999 });
    }
}
#[ocamljs_bindgen(module = Corpus)]
pub async fn drop_hook(corpus: &Corpus, input: &Input, callback: Callback<Progress>) -> u32 {
    let _hook = NotifyDrop(callback);
    let _ = Next(input.channel.clone()).await;
    corpus.revision
}
#[ocamljs_bindgen(opaque)]
pub struct Dropper {
    callback: Callback<Progress>,
}
impl Drop for Dropper {
    fn drop(&mut self) {
        let _ = self.callback.emit(Progress { records: 777 });
    }
}
#[ocamljs_bindgen(module = Dropper, name = "create")]
pub fn dropper(callback: Callback<Progress>) -> Dropper {
    Dropper { callback }
}
#[ocamljs_bindgen(module = Dropper, name = "notify")]
pub fn notify_dropper(dropper: &Dropper) -> u32 {
    let _ = dropper.callback.emit(Progress { records: 1 });
    1
}
#[ocamljs_bindgen(module = Dropper)]
pub async fn dropper_after_progress(callback: Callback<Progress>) -> Dropper {
    yield_now().await;
    let _ = callback.emit(Progress { records: 1 });
    Dropper { callback }
}
#[ocamljs_bindgen]
impl Input {
    pub async fn fetch_gate(&self, url: &str) -> Result<u32, IndexError> {
        let mut stream = ocamljs_bindgen::fetch::FetchStream::open(url);
        let _ = Next(self.channel.clone()).await;
        let mut bytes = 0;
        while let Some(chunk) = stream.next().await.map_err(|_| IndexError::Network)? {
            bytes += chunk.len() as u32;
            yield_now().await;
        }
        Ok(bytes)
    }
}

#[ocamljs_bindgen(throws, module = Index)]
pub async fn throw_owned() -> Result<(), Index> {
    yield_now().await;
    Err(Index {
        entries: Rc::new(vec!["owned failure".into()]),
    })
}

#[ocamljs_bindgen(module = Index)]
pub async fn own_index() -> Index {
    yield_now().await;
    Index {
        entries: Rc::new(vec![]),
    }
}

// A callback preceding a borrow exercises registration rollback on preflight.
#[ocamljs_bindgen(module = Corpus)]
pub fn callback_then_borrow(progress: Callback<Progress>, corpus: &Corpus) -> u32 {
    let _ = progress.emit(Progress {
        records: corpus.revision,
    });
    corpus.revision
}

#[ocamljs_bindgen::ocamljs_bindgen]
pub mod arithmetic {
    #[ocamljs_bindgen::ocamljs_bindgen]
    pub async fn twice(value: f64) -> f64 {
        value * 2.0
    }
}

#[ocamljs_bindgen(host)]
pub struct Blob;
#[ocamljs_bindgen(host)]
pub struct BlobParts;
#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(method, js_name = "text")]
    fn blob_text(
        blob: &Blob,
    ) -> Result<
        ocamljs_bindgen::host::Promise<ocamljs_bindgen::host::JsString>,
        ocamljs_bindgen::HostError,
    >;
    #[ocamljs_bindgen(getter, js_name = "size")]
    fn blob_size(blob: &Blob) -> Result<f64, ocamljs_bindgen::HostError>;
    #[ocamljs_bindgen(js_namespace = "Array", js_name = "of")]
    fn blob_parts(text: &str) -> Result<BlobParts, ocamljs_bindgen::HostError>;
    #[ocamljs_bindgen(constructor, js_name = "Blob")]
    fn new_blob(parts: &BlobParts) -> Result<Blob, ocamljs_bindgen::HostError>;
}
#[ocamljs_bindgen(module=Blob,name="of_text")]
pub fn blob(text: &str) -> Result<Blob, ocamljs_bindgen::HostError> {
    new_blob(&blob_parts(text)?)
}
/// Index a native Blob in the worker; retain its reference and progress callback
/// while browser text I/O is pending, then commit through the existing builder.
#[ocamljs_bindgen(module=Index,name="of_blob")]
pub async fn build_blob(
    corpus: &mut Corpus,
    blob: &Blob,
    progress: Callback<Progress>,
) -> Result<Index, IndexError> {
    if blob_size(blob).map_err(IndexError::Browser)? > (1024 * 1024) as f64 {
        return Err(IndexError::TooLarge);
    }
    let text = blob_text(blob)
        .map_err(IndexError::Browser)?
        .await
        .map_err(IndexError::Browser)?;
    let text = text
        .to_rust_string(1024 * 1024)
        .map_err(|error| match error {
            ocamljs_bindgen::host::CopyError::TooLarge => IndexError::TooLarge,
            ocamljs_bindgen::host::CopyError::Host(error) => IndexError::Browser(error),
        })?;
    let mut builder = IndexBuilder::default();
    builder.push(&text)?;
    while builder.batch()? {
        builder.progress(&progress);
        yield_now().await;
    }
    builder.commit(corpus)
}

#[ocamljs_bindgen]
pub type NumericValues<T> = Vec<T>;
#[ocamljs_bindgen]
pub type NumericSlice = [f64];
#[ocamljs_bindgen]
pub async fn numeric_sum_after_yield(values:NumericValues<f64>,borrowed:&NumericSlice)->f64 {
    yield_now().await;
    values.iter().chain(borrowed).sum()
}

#[ocamljs_bindgen]
pub fn numeric_notify(values:NumericValues<f64>,progress:Callback<Progress>,borrowed:&NumericSlice)->f64 {
    let _=progress.emit(Progress{records:values.len() as u32});
    values.iter().chain(borrowed).sum()
}
