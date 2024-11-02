use thiserror::Error;



#[derive(Error, Debug)]
pub enum AddPeerError {}

#[derive(Error, Debug)]
pub enum OpenChannelError {}