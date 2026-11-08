//! Browser fetch stream with bounded chunks and ordinary Rust wake semantics.
use crate::{dispose, input_owner, insert, with_ref};
use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};
#[derive(Debug)]
pub enum FetchError {
    Network,
    TooLarge,
}
impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Network => "fetch failed",
            Self::TooLarge => "response exceeds size limit",
        })
    }
}
impl std::error::Error for FetchError {}

/// Fetch UTF-8 text, with a bounded one-MiB response body.
pub async fn get_text(url: &str) -> Result<String, FetchError> {
    get_text_limited(url, 1024 * 1024).await
}

/// Collect a response while enforcing the bound before growing its allocation.
pub async fn get_text_limited(url: &str, limit: usize) -> Result<String, FetchError> {
    let mut response = FetchStream::open(url);
    let mut text = String::new();
    while let Some(chunk) = response.next().await? {
        if chunk.len() > limit.saturating_sub(text.len()) {
            return Err(FetchError::TooLarge);
        }
        text.push_str(&chunk);
    }
    Ok(text)
}
#[derive(Default)]
struct State {
    chunks: VecDeque<Rc<Vec<u8>>>,
    end: Option<bool>,
    waker: Option<Waker>,
}
struct Channel(Rc<RefCell<State>>);
crate::opaque_storage!(Channel);
pub struct FetchStream {
    id: u32,
    state: Rc<RefCell<State>>,
    utf8_pending: Vec<u8>,
}
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "ocamljs")]
unsafe extern "C" {
    fn fetch_open(id: u32, ptr: usize, len: usize);
    fn fetch_close(id: u32);
    fn fetch_credit(id: u32);
}
impl FetchStream {
    pub fn open(url: &str) -> Self {
        let state = Rc::new(RefCell::new(State::default()));
        let id = insert(Channel(state.clone()));
        #[cfg(target_arch = "wasm32")]
        unsafe {
            fetch_open(id, url.as_ptr() as usize, url.len());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = url;
            state.borrow_mut().end = Some(false);
        }
        Self {
            id,
            state,
            utf8_pending: Vec::new(),
        }
    }
    pub fn next(&mut self) -> Next<'_> {
        Next(self)
    }
}
impl Drop for FetchStream {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            fetch_close(self.id);
        }
        let _ = unsafe { dispose(self.id) };
    }
}
pub struct Next<'a>(&'a mut FetchStream);
impl Future for Next<'_> {
    type Output = Result<Option<String>, FetchError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self.get_mut().0;
        loop {
            let chunk = { this.state.borrow_mut().chunks.pop_front() };
            if let Some(chunk) = chunk {
                #[cfg(target_arch = "wasm32")]
                unsafe {
                    fetch_credit(this.id);
                }
                let mut bytes = Rc::try_unwrap(chunk).unwrap_or_else(|shared| (*shared).clone());
                if !this.utf8_pending.is_empty() {
                    let mut joined = std::mem::take(&mut this.utf8_pending);
                    joined.extend_from_slice(&bytes);
                    bytes = joined;
                }
                match String::from_utf8(bytes) {
                    Ok(text) => return Poll::Ready(Ok(Some(text))),
                    Err(error) => {
                        let utf8 = error.utf8_error();
                        if utf8.error_len().is_some() {
                            return Poll::Ready(Err(FetchError::Network));
                        }
                        let mut bytes = error.into_bytes();
                        this.utf8_pending = bytes.split_off(utf8.valid_up_to());
                        // The prefix was validated by String::from_utf8 above.
                        let text = unsafe { String::from_utf8_unchecked(bytes) };
                        if !text.is_empty() {
                            return Poll::Ready(Ok(Some(text)));
                        }
                    }
                }
            } else {
                let mut s = this.state.borrow_mut();
                if let Some(ok) = s.end {
                    return Poll::Ready(if ok && this.utf8_pending.is_empty() {
                        Ok(None)
                    } else {
                        Err(FetchError::Network)
                    });
                }
                s.waker = Some(cx.waker().clone());
                return Poll::Pending;
            }
        }
    }
}
fn channel(id: u32) -> Option<Rc<RefCell<State>>> {
    unsafe { with_ref::<Channel, _>(id, |channel| channel.0.clone()) }.ok()
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_fetch_push(id: u32, buffer: u32) -> u32 {
    let Some(channel) = channel(id) else { return 0 };
    let wake = {
        let mut s = channel.borrow_mut();
        if s.end.is_some() {
            return 0;
        }
        if s.chunks.len() >= 4 {
            return 2;
        }
        let Ok(bytes) = input_owner(buffer) else {
            return 0;
        };
        if bytes.len() > 32768 {
            return 0;
        }
        s.chunks.push_back(bytes);
        s.waker.take()
    };
    if let Some(w) = wake {
        w.wake();
    }
    1
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_fetch_end(id: u32, ok: u32) {
    if let Some(channel) = channel(id) {
        let wake = {
            let mut s = channel.borrow_mut();
            s.end = Some(ok != 0);
            s.waker.take()
        };
        if let Some(w) = wake {
            w.wake();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stream(chunks: &[&[u8]]) -> FetchStream {
        let state = Rc::new(RefCell::new(State {
            chunks: chunks.iter().map(|b| Rc::new(b.to_vec())).collect(),
            end: Some(true),
            waker: None,
        }));
        let id = insert(Channel(state.clone()));
        FetchStream {
            id,
            state,
            utf8_pending: Vec::new(),
        }
    }
    fn collect(mut stream: FetchStream) -> Result<String, FetchError> {
        let mut context = Context::from_waker(Waker::noop());
        let mut text = String::new();
        loop {
            match Pin::new(&mut stream.next()).poll(&mut context) {
                Poll::Ready(Ok(Some(chunk))) => text.push_str(&chunk),
                Poll::Ready(Ok(None)) => return Ok(text),
                Poll::Ready(Err(error)) => return Err(error),
                Poll::Pending => panic!("fixture has already ended"),
            }
        }
    }
    #[test]
    fn utf8_survives_every_chunk_boundary_without_changing_the_text() {
        let text = "\u{feff}aλ🦀z";
        for at in 0..=text.len() {
            assert_eq!(
                collect(stream(&[&text.as_bytes()[..at], &text.as_bytes()[at..]])).unwrap(),
                text
            );
        }
        let bytes = text
            .as_bytes()
            .iter()
            .map(std::slice::from_ref)
            .collect::<Vec<_>>();
        assert_eq!(collect(stream(&bytes)).unwrap(), text);
    }
    #[test]
    fn malformed_and_unfinished_utf8_fail() {
        for chunks in [
            vec![&b"\xff"[..]],
            vec![&b"\xf0\x9f"[..]],
            vec![&b"\xce"[..], &b"x"[..]],
        ] {
            assert!(matches!(collect(stream(&chunks)), Err(FetchError::Network)));
        }
    }
}
