use std::task::ready;
use aes::{
    Aes128,
    cipher::{
        KeyIvInit, 
        AsyncStreamCipher
    }
};
use cfb8::{
    Encryptor, Decryptor,
};
use tokio::io::{AsyncRead, AsyncWrite};
use std::cell::RefCell;

pub struct EncryptedStream<S> {
    pub stream: S,
    pub encryptor: RefCell<Encryptor<Aes128>>,
    pub decryptor: RefCell<Decryptor<Aes128>>,
}

impl<S> EncryptedStream<S> {
    pub fn new(stream: S, secret: &[u8]) -> Self {
        dbg!(secret.len());
        let encryptor = Encryptor::<Aes128>::new_from_slices(secret, secret).unwrap();
        let decryptor = Decryptor::<Aes128>::new_from_slices(secret, secret).unwrap();
        EncryptedStream {
            stream,
            encryptor: RefCell::new(encryptor),
            decryptor: RefCell::new(decryptor),
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for EncryptedStream<S> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let filled_before = buf.filled().len();
        let this = self.as_mut().get_mut();
        let poll = std::pin::Pin::new(&mut this.stream).poll_read(cx, buf);
        
        if let std::task::Poll::Ready(Ok(())) = poll {
            let filled_after = buf.filled().len();
            if filled_after > filled_before {
                let new_data = &mut buf.filled_mut()[filled_before..filled_after];
                
                // Borrow mutably and decrypt in place
                this.decryptor.borrow_mut().decrypt(new_data.into());
            }
        }
        
        poll
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for EncryptedStream<S> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut encrypted_buf = buf.to_vec();
        let this = self.as_mut().get_mut();
        
        // Borrow mutably and encrypt in place
        this.encryptor.borrow_mut().encrypt(encrypted_buf.as_mut_slice().into());
        
        let n = ready!(std::pin::Pin::new(&mut this.stream).poll_write(cx, &encrypted_buf))?;
        std::task::Poll::Ready(Ok(n))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>, 
        cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>, 
        cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}