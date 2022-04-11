use thiserror::Error;

// TODO: Document errors and their variants.

#[non_exhaustive]
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

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum IoError {
    #[error("Failed to open patch for unknown reason.")]
    FailedToOpenPatch,
    #[error("Failed to close patch, because the handle which was provided is null.")]
    FailedToClosePatch,
    #[error("Failed to open gui, please provide a valid path to the pd binary.")]
    FailedToOpenGui,
    #[error("The path you have provided does not exist in the file system. Path: {0}")]
    PathDoesNotExist(String),
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("No destination found for receiver: `{0}` in loaded pd patch.")]
    MissingDestination(String),
    #[error("Values which are being sent are out of range.")]
    OutOfRange,
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SubscriptionError {
    #[error("Failed to subscribe to sender: `{0}` in loaded pd patch.")]
    FailedToSubscribeToSender(String),
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SizeError {
    #[error("The maximum size specified is too large.")]
    TooLarge,
    #[error("Could not determine the size.")]
    CouldNotDetermine,
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ArrayError {
    #[error("The array which you're trying to access doesn't exist.")]
    NonExistent,
    #[error("The position in array which you're trying to write is out of bounds.")]
    OutOfBounds,
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum LibpdError {
    #[error("")]
    InitializationError(InitializationError),
    #[error("")]
    IoError(IoError),
    #[error("")]
    CommunicationError(CommunicationError),
    #[error("")]
    SubscriptionError(SubscriptionError),
    #[error("")]
    SizeError(SizeError),
    #[error("")]
    ArrayError(ArrayError),
}
