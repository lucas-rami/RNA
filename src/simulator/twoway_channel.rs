// Standard library
use std::marker::PhantomData;
use std::sync::mpsc::{self, Receiver, Sender};

// pub trait EndpointRole {}
// pub trait Master: EndpointRole {}
// pub trait Slave: EndpointRole {}
// pub struct MasterRole {}
// impl EndpointRole for MasterRole {}
// impl Master for MasterRole {}
// pub struct SlaveRole {}
// impl EndpointRole for SlaveRole {}
// impl Slave for SlaveRole {}

pub fn twoway_channel<T, R>(
) -> (MasterEndpoint<MessageType<T>, R>, SlaveEndpoint<R, MessageType<T>>) {
    let (master_tx, slave_rx) = mpsc::channel();
    let (slave_tx, master_rx) = mpsc::channel();
    (
        MasterEndpoint::new(master_tx, master_rx),
        SlaveEndpoint::new(slave_tx, slave_rx),
    )
}

pub struct MasterEndpoint<T, R> {
    tx: Sender<MessageType<T>>,
    rx: Receiver<R>,
}

impl<T, R> MasterEndpoint<T, R> {
    fn new(tx: Sender<MessageType<T>>, rx: Receiver<R>) -> Self {
        Self {
            tx,
            rx,
        }
    }

    pub fn send_and_expect_response(&self, request: T) -> R {
        if let Err(_) = self.tx.send(MessageType::ResponseRequired(request)) {
            panic!(ERR_DEAD_SLAVE)
        }

        match self.rx.recv() {
            Ok(response) => return response,
            Err(_) => panic!(ERR_DEAD_SLAVE),
        }
    }

    pub fn create_third_party(&self) -> DirectionalEndpoint<T> {
        DirectionalEndpoint::new(self.tx.clone())
    }
}

pub struct SlaveEndpoint<T, R> {
    tx: Sender<T>,
    rx: Receiver<MessageType<R>>,
}

impl<T, R> SlaveEndpoint<T, R> {
    fn new(tx: Sender<T>, rx: Receiver<MessageType<R>>) -> Self {
        Self {
            tx,
            rx,
        }
    }

    pub fn wait_for_mail(&self) -> MessageType<T> {
        match self.rx.recv() {
            Ok(request) => 
            Err(_) => MessageType::DeadChannel,
        }
    }
}

pub struct DirectionalEndpoint<T> {
    tx: Sender<MessageType<T>>,
}

impl<T> DirectionalEndpoint<T> {
    fn new(tx: Sender<MessageType<T>>) -> Self {
        Self { tx }
    }

    pub fn send(&self, msg: T) -> bool {
        match self.tx.send(MessageType::SimpleMsg(msg)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

pub struct Request<'a, T, R> {
    tx: &'a Sender<T>,
    request: R,
}

impl<'a, T, R> Request<'a, T, R> {
    fn new(tx: &'a Sender<T>, request: R) -> Self {
        Self {tx, request}
    }

    pub fn respond(self, response: T) -> bool {
        match self.tx.send(response) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

enum MessageType<T> {
    ResponseRequired(T),
    SimpleMsg(T),
}



const ERR_DEAD_SLAVE: &str = "Slave endpoint died before master endpoint.";
