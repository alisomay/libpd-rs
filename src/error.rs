use thiserror::Error;

// TODO: Document errors and their variants.

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
    #[error("Values which are being sent are out of range.")]
    OutOfRange,
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
pub enum ArrayError {
    #[error("The array which you're trying to access doesn't exist.")]
    NonExistent,
    #[error("The position in array which you're trying to write is out of bounds.")]
    OutOfBounds,
    // Add more errors here..
}

// TODO: Renew this error explanations.
#[derive(Error, Debug)]
pub enum LibpdError {
    #[error("Error in initialization.")]
    InitializationError(InitializationError),
    #[error("Error in I/O.")]
    IoError(IoError),
    #[error("Error in accessing files.")]
    FileSystemError(FileSystemError),
    #[error("Error in sending data to pd.")]
    SendError(SendError),
    #[error("Error in subscribing pd senders.")]
    SubscriptionError(SubscriptionError),
    #[error("Error in sizes.")]
    SizeError(SizeError),
    #[error("Error in array.")]
    ArrayError(ArrayError),
}
