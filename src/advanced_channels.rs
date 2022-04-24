// Standard library
use std::sync::mpsc::{self, Receiver, RecvError, Sender};

pub trait TransmittingEnd {
    type MSG;

    fn send(&self, msg: Self::MSG);
}

pub fn twoway_channel<T, R>() -> (MasterEndpoint<T, R>, SlaveEndpoint<R, T>) {
    let (master_tx, slave_rx) = mpsc::channel();
    let (slave_tx, master_rx) = mpsc::channel();
    (
        MasterEndpoint::new(master_tx, master_rx),
        SlaveEndpoint::new(slave_tx, slave_rx),
    )
}

pub fn oneway_channel<T>() -> (SimpleSender<T>, SimpleReceiver<T>) {
    let (tx, rx) = mpsc::channel();
    (SimpleSender::new(tx), SimpleReceiver::new(rx))
}

pub struct MasterEndpoint<T, R> {
    tx: Sender<MessageType<T>>,
    rx: Receiver<R>,
}

impl<T, R> MasterEndpoint<T, R> {
    fn new(tx: Sender<MessageType<T>>, rx: Receiver<R>) -> Self {
        Self { tx, rx }
    }

    pub fn send_and_wait_for_response(&self, request: T) -> R {
        self.send_raw(MessageType::Message(request, true));

        match self.rx.recv() {
            Ok(response) => response,
            Err(_) => panic!("{}", ERR_DEAD_SLAVE),
        }
    }

    fn send_raw(&self, msg: MessageType<T>) {
        if let Err(_) = self.tx.send(msg) {
            panic!("{}", ERR_DEAD_SLAVE);
        }
    }

    pub fn create_third_party(&self) -> ThirdPartySender<T> {
        ThirdPartySender::new(self.tx.clone())
    }

    pub fn close(self) {
        self.send_raw(MessageType::DeadChannel);
    }
}

impl<T, R> TransmittingEnd for MasterEndpoint<T, R> {
    type MSG = T;

    fn send(&self, msg: Self::MSG) {
        self.send_raw(MessageType::Message(msg, false));
    }
}

impl<T, R> Drop for MasterEndpoint<T, R> {
    fn drop(&mut self) {
        self.send_raw(MessageType::DeadChannel);
    }
}

pub struct SlaveEndpoint<T, R> {
    tx: Sender<T>,
    rx: Receiver<MessageType<R>>,
}

impl<T, R> SlaveEndpoint<T, R> {
    fn new(tx: Sender<T>, rx: Receiver<MessageType<R>>) -> Self {
        Self { tx, rx }
    }

    pub fn wait_for_mail<'a>(&'a self) -> MailType<'a, T, R> {
        match self.rx.recv() {
            Ok(msg) => match msg {
                MessageType::Message(msg, true) => {
                    MailType::Message(msg, Some(Request::new(&self.tx)))
                }
                MessageType::Message(msg, false) => MailType::Message(msg, None),
                MessageType::DeadChannel => MailType::DeadChannel,
            },
            Err(_) => MailType::DeadChannel,
        }
    }

    pub fn wait_for_msg(&self) -> R {
        match self.rx.recv() {
            Ok(msg) => match msg {
                MessageType::Message(msg, false) => msg,
                _ => panic!("{}", ERR_DEAD_MASTER),
            },
            Err(_) => panic!("{}", ERR_DEAD_MASTER),
        }
    }
}

pub struct Request<'a, T> {
    tx: &'a Sender<T>,
    is_answered: bool,
}

impl<'a, T> Request<'a, T> {
    fn new(tx: &'a Sender<T>) -> Self {
        Self {
            tx,
            is_answered: false,
        }
    }

    pub fn respond(mut self, response: T) {
        if let Err(_) = self.tx.send(response) {
            panic!("{}", ERR_DEAD_MASTER);
        }
        self.is_answered = true;
    }
}

impl<'a, T> Drop for Request<'a, T> {
    fn drop(&mut self) {
        if !self.is_answered {
            panic!("{}", ERR_NO_RESPONSE);
        }
    }
}

pub struct ThirdPartySender<T> {
    tx: Sender<MessageType<T>>,
}

impl<T> ThirdPartySender<T> {
    fn new(tx: Sender<MessageType<T>>) -> Self {
        Self { tx }
    }
}

impl<T> TransmittingEnd for ThirdPartySender<T> {
    type MSG = T;

    fn send(&self, msg: Self::MSG) {
        let _ = self.tx.send(MessageType::Message(msg, false));
    }
}

pub struct SimpleSender<T> {
    tx: Sender<T>,
}

impl<T> SimpleSender<T> {
    fn new(tx: Sender<T>) -> Self {
        Self { tx }
    }
}

impl<T> TransmittingEnd for SimpleSender<T> {
    type MSG = T;

    fn send(&self, msg: Self::MSG) {
        if let Err(_) = self.tx.send(msg) {
            panic!("{}", ERR_DEAD_SLAVE);
        }
    }
}

pub struct SimpleReceiver<R> {
    rx: Receiver<R>,
}

impl<R> SimpleReceiver<R> {
    fn new(rx: Receiver<R>) -> Self {
        Self { rx }
    }

    pub fn wait_for_mail(&self) -> Result<R, RecvError> {
        self.rx.recv()
    }
}

enum MessageType<T> {
    Message(T, bool),
    DeadChannel,
}

pub enum MailType<'a, T, R> {
    Message(R, Option<Request<'a, T>>),
    DeadChannel,
}

const ERR_DEAD_MASTER: &str =
    "Master endpoint died before slave endpoint could respond to request.";
const ERR_DEAD_SLAVE: &str = "Slave endpoint died before master endpoint.";
const ERR_NO_RESPONSE: &str = "Request object was dropped before sending a response.";
