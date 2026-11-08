use ocamljs_bindgen::{Callback, ocamljs_bindgen, yield_now};
use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{Poll, Waker},
};
#[ocamljsify::ocamljsify]
#[derive(Debug)]
pub enum IndexError {
    InvalidRecord,
    TooLarge,
    Closed,
    Full,
    Network,
    Cancelled,
    Browser(ocamljs_bindgen::HostError),
}
#[ocamljsify::ocamljsify]
pub struct Progress {
    pub records: u32,
}
#[ocamljs_bindgen(opaque)]
pub struct Corpus {
    index: Rc<Vec<String>>,
    revision: u32,
}
#[ocamljs_bindgen(opaque)]
pub struct Index {
    entries: Rc<Vec<String>>,
}
#[ocamljs_bindgen(module = Corpus, name = "create")]
pub fn corpus() -> Corpus {
    Corpus {
        index: Rc::new(vec![]),
        revision: 0,
    }
}
#[ocamljs_bindgen]
pub fn live_entries() -> u32 {
    ocamljs_bindgen::__ocamljs_live() as u32
}
// Transport-independent NDJSON parser. Each async poll parses at most 16 lines;
// compaction occurs only when a new chunk arrives, never once per record.
#[derive(Default)]
struct IndexBuilder {
    entries: Vec<String>,
    pending: String,
    offset: usize,
    total: usize,
    next_progress: usize,
}
impl IndexBuilder {
    fn progress(&mut self, progress: &Callback<Progress>) {
        let next = self.next_progress.max(16);
        if self.entries.len() >= next && next <= 2048 {
            let _ = progress.emit(Progress {
                records: self.entries.len() as u32,
            });
            self.next_progress = next * 2;
        }
    }
    fn line(entries: &mut Vec<String>, line: &str) -> Result<(), IndexError> {
        if line.trim().is_empty() {
            return Ok(());
        }
        if !line.trim_start().starts_with('{') {
            return Err(IndexError::InvalidRecord);
        }
        if entries.len() >= 4096 {
            return Err(IndexError::TooLarge);
        }
        #[derive(serde::Deserialize)]
        struct Record {
            text: String,
        }
        let record: Record = serde_json::from_str(line).map_err(|_| IndexError::InvalidRecord)?;
        entries.push(record.text);
        Ok(())
    }
    fn push(&mut self, chunk: &str) -> Result<(), IndexError> {
        self.total += chunk.len();
        if self.total > 1024 * 1024 {
            return Err(IndexError::TooLarge);
        }
        if self.offset > 0 {
            self.pending.drain(..self.offset);
            self.offset = 0;
        }
        self.pending.push_str(chunk);
        Ok(())
    }
    fn batch(&mut self) -> Result<bool, IndexError> {
        let mut consumed = 0;
        for _ in 0..16 {
            let Some(end) = self.pending[self.offset..]
                .find('\n')
                .map(|n| n + self.offset)
            else {
                break;
            };
            Self::line(&mut self.entries, &self.pending[self.offset..end])?;
            self.offset = end + 1;
            consumed += 1;
        }
        Ok(consumed > 0)
    }
    fn commit(mut self, corpus: &mut Corpus) -> Result<Index, IndexError> {
        Self::line(&mut self.entries, &self.pending[self.offset..])?;
        let revision = corpus.revision.checked_add(1).ok_or(IndexError::TooLarge)?;
        let entries = Rc::new(self.entries);
        corpus.index = Rc::clone(&entries);
        corpus.revision = revision;
        Ok(Index { entries })
    }
}
#[ocamljs_bindgen(module = Index)]
pub fn query(index: &Index, needle: &str) -> Vec<String> {
    index
        .entries
        .iter()
        .filter(|s| s.contains(needle))
        .take(128)
        .cloned()
        .collect()
}
// Application cancellation: ordinary Rust state, separate from the corpus lease.
#[derive(Default)]
struct CancelState {
    cancelled: bool,
    started: bool,
    waker: Option<Waker>,
}
#[ocamljs_bindgen(opaque)]
pub struct SearchCancel {
    state: Rc<RefCell<CancelState>>,
}
#[ocamljs_bindgen(module = SearchCancel, name = "create")]
pub fn search_cancel() -> SearchCancel {
    SearchCancel {
        state: Rc::default(),
    }
}
#[ocamljs_bindgen(module = SearchCancel)]
pub fn cancel(search: &SearchCancel) {
    let wake = {
        let mut state = search.state.borrow_mut();
        state.cancelled = true;
        state.waker.take()
    };
    if let Some(wake) = wake {
        wake.wake();
    }
}
impl SearchCancel {
    fn check(&self) -> Result<(), IndexError> {
        if self.state.borrow().cancelled {
            Err(IndexError::Cancelled)
        } else {
            Ok(())
        }
    }
    async fn until_cancelled<F: Future>(&self, future: F) -> Result<F::Output, IndexError> {
        let mut future = std::pin::pin!(future);
        std::future::poll_fn(|cx| {
            {
                let mut state = self.state.borrow_mut();
                if state.cancelled {
                    return Poll::Ready(Err(IndexError::Cancelled));
                }
                state.waker = Some(cx.waker().clone());
            }
            future.as_mut().poll(cx).map(Ok)
        })
        .await
    }
}

#[ocamljs_bindgen(module = Index)]
pub async fn build_url(
    corpus: &mut Corpus,
    url: &str,
    cancellation: &SearchCancel,
    progress: Callback<Progress>,
) -> Result<Index, IndexError> {
    {
        let mut state = cancellation.state.borrow_mut();
        if state.started {
            return Err(IndexError::Closed);
        }
        state.started = true;
    }
    cancellation.check()?;
    let mut builder = IndexBuilder::default();
    let mut stream = ocamljs_bindgen::fetch::FetchStream::open(url);
    while let Some(chunk) = cancellation
        .until_cancelled(stream.next())
        .await?
        .map_err(|_| IndexError::Network)?
    {
        cancellation.check()?;
        builder.push(&chunk)?;
        while builder.batch()? {
            cancellation.check()?;
            builder.progress(&progress);
            yield_now().await;
        }
    }
    cancellation.check()?;
    builder.commit(corpus)
}

// Additional cases exercised by the worker lifetime-validation command.
include!("../../tests/rust/conformance.rs");

#[cfg(test)]
#[path = "../../tests/rust/parser.rs"]
mod parser_tests;
