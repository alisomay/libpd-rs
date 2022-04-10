use thiserror::Error;

// TODO: Document errors.

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
    #[error("Failed to open gui, please provide a valid path to the pd binary.")]
    FailedToOpenGui,
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
    #[error("Couldn't determine the size.")]
    CouldNotDetermine,
    // Add more errors here..
}

#[derive(Error, Debug)]
pub enum LibpdError {
    #[error("Error in initialization.")]
    InitializationError(InitializationError),
    #[error("Error in initialization.")]
    IoError(IoError),
    #[error("Error in initialization.")]
    FileSystemError(FileSystemError),
    #[error("Error in initialization.")]
    SendError(SendError),
    #[error("Error in initialization.")]
    SubscriptionError(SubscriptionError),
    #[error("Error in initialization.")]
    SizeError(SizeError),
}
