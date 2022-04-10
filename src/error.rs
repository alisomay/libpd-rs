use thiserror::Error;

// TODO: Document errors.

#[derive(Debug)]
pub enum LibpdError {
    InitializationError,
    IoError,
    FileSystemError,
    SendError,
    SubscriptionError,
    SizeError,
}

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Pure Data is already initialized.")]
    AlreadyInitialized,
    #[error("Failed to initialize ring buffers which are needed for the message queue.")]
    RingBufferInitializationError,
    #[error("An unknown error occurred in Pure Data initialization.")]
    InitializationFailed,
    #[error("An unknown error occurred in Pure Data audio initialization.")]
    AudioInitializationFailed,
}

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Failed to open patch for unknown reason.")]
    FailedToOpenPatch,
    #[error("Failed to close patch, because the handle provided is null.")]
    FailedToClosePatch,
    // Add more errors here..
}

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("The path you have provided does not exist in the file system. Path: {0}")]
    PathDoesNotExist(String),
    // Add more errors here..
}

#[derive(Error, Debug)]
pub enum SendError {
    #[error("No destination found for receiver: `{0}` in loaded pd patch.")]
    MissingDestination(String),
    // Add more errors here..
}

#[derive(Error, Debug)]
pub enum SubscriptionError {
    #[error("Failed to subscribe to sender: `{0}` in loaded pd patch.")]
    FailedToSubscribeToSender(String),
    // Add more errors here..
}

#[derive(Error, Debug)]
pub enum SizeError {
    #[error("The maximum size specified is too large.")]
    TooLarge,
    // Add more errors here..
}
