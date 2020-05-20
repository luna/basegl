//! Module defining types representing messages being sent between client and server.

use crate::prelude::*;

use crate::language_server::Path as LSPath;

use json_rpc::Transport;



// ===============
// === Aliases ===
// ===============

/// An owning representation of the message received from a server.
pub type MessageFromServerOwned = MessageFromServer<FromServerPayloadOwned>;

/// An owning representation of the message received from a server.
pub type MessageToServerOwned = MessageToServer<ToServerPayloadOwned>;

/// An non-owning representation of the message to be sent to the server.
pub type MessageToServerRef<'a> = MessageToServer<ToServerPayload<'a>>;



// ================
// === Newtypes ===
// ================

/// A message sent from client to server (`InboundMessage` in the spec).
#[derive(Clone,Debug,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct MessageToServer<T>(pub Message<T>);

impl<T> MessageToServer<T> {
    /// Wraps the given payload into a message envelope. Generates a unique ID for the message.
    pub fn new(payload:T) -> Self {
        Self(Message::new(payload))
    }
}

/// A message sent from server to client (`OutboundMessage` in the spec).
#[derive(Clone,Debug,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct MessageFromServer<T>(pub Message<T>);

impl<T> MessageFromServer<T> {
    /// Wraps the given payload into a message envelope. Generates a unique ID for the message.
    pub fn new(payload:T) -> Self {
        Self(Message::new(payload))
    }
}



// =============
// === Types ===
// =============

/// Identifies the visualization in the update message.
#[allow(missing_docs)]
#[derive(Clone,Debug,Copy,PartialEq)]
pub struct VisualisationContext {
    pub visualization_id : Uuid,
    pub context_id       : Uuid,
    pub expression_id    : Uuid,
}



// ================
// === Payloads ===
// ================

#[allow(missing_docs)]
#[derive(Clone,Debug,PartialEq)]
pub enum ToServerPayloadOwned {
    InitSession {client_id:Uuid},
    WriteFile   {path:LSPath, contents:Vec<u8>},
    ReadFile    {path:LSPath}
}

#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub enum FromServerPayloadOwned {
    Error {code:i32, message:String},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:Vec<u8>},
    FileContentsReply   {contents:Vec<u8>},
}

#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub enum ToServerPayload<'a> {
    InitSession {client_id:Uuid},
    WriteFile   {path:&'a LSPath, contents:&'a[u8]},
    ReadFile    {path:&'a LSPath}
}

#[allow(missing_docs)]
#[derive(Clone,Debug)]
pub enum FromServerPayload<'a> {
    Error {code:i32, message:&'a str},
    Success {},
    VisualizationUpdate {context:VisualisationContext, data:&'a [u8]},
    FileContentsReply {contents:&'a [u8]},
}



// ===============
// === Message ===
// ===============

/// Common message envelope for binary protocol.
///
/// `T` should represent the payload.
#[derive(Clone,Debug)]
pub struct Message<T> {
    /// Each message bears unique id.
    pub message_id     : Uuid,
    /// When sending reply, server sets this to the request's `message_id`.
    pub correlation_id : Option<Uuid>,
    #[allow(missing_docs)]
    pub payload        : T,
}

impl<T> Message<T> {
    /// Wraps the given payload into a message envelope. Generates a unique ID for the message.
    /// Private, as users should use either `MessageToServer::new` or `MessageFromServer::new`.
    fn new(payload:T) -> Message<T> {
        Message {
            message_id     : Uuid::new_v4(),
            correlation_id : None,
            payload,
        }
    }
}

impl<'a> crate::handler::IsRequest for MessageToServerRef<'a> {
    type Id = Uuid;

    fn send(&self, transport:&mut dyn Transport) -> FallibleResult<()> {
        self.with_serialized(|data| transport.send_binary(data))
    }

    fn id(&self) -> Self::Id {
        self.message_id
    }
}
