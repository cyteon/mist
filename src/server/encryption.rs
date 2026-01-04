use std::task::ready;
use aes::Aes128;
use aes::cipher::{KeyIvInit, BlockEncryptMut, BlockDecryptMut, generic_array::GenericArray};
use cfb8::{Encryptor, Decryptor};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub struct EncryptedStream<S> {
    pub stream: S,
    pub encryptor: Encryptor<Aes128>,
    pub decryptor: Decryptor<Aes128>,
}

pub struct EncryptedReader<R> {
    pub stream: R,
    pub decryptor: Decryptor<Aes128>,
}

pub struct EncryptedWriter<W> {
    pub stream: W,
    pub encryptor: Encryptor<Aes128>,
}


impl<S> EncryptedStream<S> {
    pub fn new(stream: S, secret: &[u8]) -> Self {
        let encryptor = Encryptor::<Aes128>::new_from_slices(secret, secret).unwrap();
        let decryptor = Decryptor::<Aes128>::new_from_slices(secret, secret).unwrap();

        EncryptedStream {
            stream,
            encryptor,
            decryptor,
        }
    }
}

impl EncryptedStream<tokio::net::TcpStream> {
    pub fn into_split(self) -> (EncryptedReader<OwnedReadHalf>, EncryptedWriter<OwnedWriteHalf>) {
        let (read, write) = self.stream.into_split();

        (
            EncryptedReader {
                stream: read,
                decryptor: self.decryptor,
            },

            EncryptedWriter {
                stream: write,
                encryptor: self.encryptor,
            }
        )
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for EncryptedReader<R> {
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
                
                for byte in new_data.iter_mut() {
                    let block = GenericArray::from_mut_slice(std::slice::from_mut(byte));
                    this.decryptor.decrypt_block_mut(block);
                }
            }
        }
        
        poll
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for EncryptedWriter<W> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut encrypted_buf = buf.to_vec();
        let this = self.as_mut().get_mut();
        
        for byte in encrypted_buf.iter_mut() {
            let block = GenericArray::from_mut_slice(std::slice::from_mut(byte));
            this.encryptor.encrypt_block_mut(block);
        }
        
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
                
                for byte in new_data.iter_mut() {
                    let block = GenericArray::from_mut_slice(std::slice::from_mut(byte));
                    this.decryptor.decrypt_block_mut(block);
                }
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
        
        for byte in encrypted_buf.iter_mut() {
            let block = GenericArray::from_mut_slice(std::slice::from_mut(byte));
            this.encryptor.encrypt_block_mut(block);
        }
        
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