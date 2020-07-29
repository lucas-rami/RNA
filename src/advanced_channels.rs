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
        self.send_raw(MessageType::ResponseRequired(request));

        match self.rx.recv() {
            Ok(response) => response,
            Err(_) => panic!(ERR_DEAD_SLAVE),
        }
    }

    fn send_raw(&self, msg: MessageType<T>) {
        if let Err(_) = self.tx.send(msg) {
            panic!(ERR_DEAD_SLAVE);
        }
    }

    pub fn create_third_party(&self) -> ThirdPartySender<T> {
        ThirdPartySender::new(self.tx.clone())
    }
}

impl<T, R> TransmittingEnd for MasterEndpoint<T, R> {
    type MSG = T;

    fn send(&self, msg: Self::MSG) {
        self.send_raw(MessageType::SimpleMsg(msg));
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
                MessageType::ResponseRequired(req) => {
                    MailType::ResponseRequired(Request::new(&self.tx, req))
                }
                MessageType::SimpleMsg(msg) => MailType::SimpleMsg(msg),
                MessageType::DeadChannel => MailType::DeadChannel,
            },
            Err(_) => MailType::DeadChannel,
        }
    }

    pub fn wait_for_simple_msg(&self) -> R {
        match self.rx.recv() {
            Ok(msg) => match msg {
                MessageType::SimpleMsg(msg) => msg,
                _ => panic!(ERR_DEAD_MASTER),
            },
            Err(_) => panic!(ERR_DEAD_MASTER),
        }
    }
}

pub struct Request<'a, T, R> {
    tx: &'a Sender<T>,
    request: R,
    is_answered: bool,
}

impl<'a, T, R> Request<'a, T, R> {
    fn new(tx: &'a Sender<T>, request: R) -> Self {
        Self {
            tx,
            request,
            is_answered: false,
        }
    }

    pub fn respond(mut self, response: T) {
        if let Err(_) = self.tx.send(response) {
            panic!(ERR_DEAD_MASTER);
        }
        self.is_answered = true;
    }

    pub fn get_request(&self) -> &R {
        &self.request
    }
}

impl<'a, T, R> Drop for Request<'a, T, R> {
    fn drop(&mut self) {
        if !self.is_answered {
            panic!(ERR_NO_RESPONSE);
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
        let _ = self.tx.send(MessageType::SimpleMsg(msg));
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
            panic!(ERR_DEAD_SLAVE);
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
    ResponseRequired(T),
    SimpleMsg(T),
    DeadChannel,
}

pub enum MailType<'a, T, R> {
    ResponseRequired(Request<'a, T, R>),
    SimpleMsg(R),
    DeadChannel,
}

const ERR_DEAD_MASTER: &str =
    "Master endpoint died before slave endpoint could repsond to request.";
const ERR_DEAD_SLAVE: &str = "Slave endpoint died before master endpoint.";
const ERR_NO_RESPONSE: &str = "Request object was dropped before sending a response.";
