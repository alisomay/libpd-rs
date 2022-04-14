// TODO: Revisit error handling completely.

use thiserror::Error;

/// Errors related to initialization.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum InitializationError {
    /// pd could not be initialized two times.
    #[error("Pure Data is already initialized.")]
    AlreadyInitialized,
    /// Error in initializing ring buffers.
    #[error("Failed to initialize ring buffers which are needed for the message queue.")]
    RingBufferInitializationError,
    /// A failure happened in pd initialization with an unknown reason.
    #[error("An unknown error occurred in Pure Data initialization.")]
    InitializationFailed,
    /// A failure happened in pd audio initialization with an unknown reason.
    #[error("An unknown error occurred in Pure Data audio initialization.")]
    AudioInitializationFailed,
}

/// Errors related to io and filesystem access.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum IoError {
    /// Failed to open patch for unknown reason.
    #[error("Failed to open patch for unknown reason.")]
    FailedToOpenPatch,
    /// Failed to close patch, because the handle which was provided is null.
    #[error("Failed to close patch, because the handle which was provided is null.")]
    FailedToClosePatch,
    /// The patch which is trying to be communicated with is not open.
    #[error("The patch which is trying to be communicated with is not open.")]
    PatchIsNotOpen,
    /// Failed to open gui, most probably because the path is invalid to the pd binary.
    #[error("Failed to open gui, please provide a valid path to the pd binary.")]
    FailedToOpenGui,
    /// The path to the patch which are being tried to open is invalid.
    #[error("The path you have provided does not exist in the file system. Path: {0}")]
    PathDoesNotExist(String),
}

/// Errors related communication between the rust app and pd.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum CommunicationError {
    /// The pd patch does not contain a receiver with the name you provided.
    #[error("No destination found for receiver: `{0}` in loaded pd patch.")]
    MissingDestination(String),
    /// A general error when the values which you are sending to the receiver are out of range.
    #[error("Values which are being sent are out of range.")]
    OutOfRange,
}

/// Errors related to subscription to senders in a pd patch.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SubscriptionError {
    /// A failure of subscription to a sender with an unknown reason.
    #[error("Failed to subscribe to sender: `{0}` in loaded pd patch.")]
    FailedToSubscribeToSender(String),
}

/// Errors related to sizes of entities.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SizeError {
    /// The size of the entity is too large.
    #[error("The maximum size specified is too large.")]
    TooLarge,
    /// Could not determine the size of the entity.
    #[error("Could not determine the size.")]
    CouldNotDetermine,
}

/// Errors related to pd arrays.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ArrayError {
    /// The array which is being tried to be accessed doesn't exist.
    #[error("The array which you're trying to access doesn't exist.")]
    NonExistent,
    /// The position in the array which is tried to be written is out of bounds.
    #[error("The position in array which you're trying to write is out of bounds.")]
    OutOfBounds,
}

pub trait LibpdError: std::error::Error + Send + Sync + 'static {}

macro_rules! impl_from_error {
    ($from:ty) => {
        impl From<$from> for Box<dyn LibpdError> {
            fn from(e: $from) -> Self {
                Box::new(e)
            }
        }
    };
}

impl LibpdError for InitializationError {}
impl LibpdError for IoError {}
impl LibpdError for CommunicationError {}
impl LibpdError for SubscriptionError {}
impl LibpdError for SizeError {}
impl LibpdError for ArrayError {}

impl_from_error!(InitializationError);
impl_from_error!(IoError);
impl_from_error!(CommunicationError);
impl_from_error!(SubscriptionError);
impl_from_error!(SizeError);
impl_from_error!(ArrayError);

// The umbrella error type for all errors in this crate.
// #[non_exhaustive]
// #[derive(Error, Debug)]
// pub enum LibpdError {
//     /// Errors related to initialization.
//     #[error("")]
//     InitializationError(InitializationError),
//     /// Errors related to io and filesystem access.
//     #[error("")]
//     IoError(IoError),
//     /// Errors related communication between the rust app and pd.
//     #[error("")]
//     CommunicationError(CommunicationError),
//     /// Errors related to subscription to senders in a pd patch.
//     #[error("")]
//     SubscriptionError(SubscriptionError),
//     /// Errors related to sizes of entities.
//     #[error("")]
//     SizeError(SizeError),
//     /// Errors related to pd arrays.
//     #[error("")]
//     ArrayError(ArrayError),
// }
