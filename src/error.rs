use std::ffi::NulError;

use thiserror::Error;

#[expect(dead_code, reason = "We might use this in the future.")]
pub(crate) const C_STRING_FAILURE: &str =
    "Provided an invalid CString, check if your string contains null bytes in the middle.";
pub(crate) const C_STR_FAILURE: &str = "Converting a CStr to an &str is failed.";

/// The umbrella error type for libpd-rs.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum PdError {
    /// An error occurred during initialization.
    #[error(transparent)]
    InitializationError(#[from] InitializationError),
    /// An error occurred during audio initialization.
    #[error(transparent)]
    AudioInitializationError(#[from] AudioInitializationError),
    /// An error occurred during patch lifecycle.
    #[error(transparent)]
    PatchLifeCycleError(#[from] PatchLifeCycleError),
    /// An error occurred during gui lifecycle.
    #[error(transparent)]
    GuiLifeCycleError(#[from] GuiLifeCycleError),
    /// An error occurred during general filesystem access.
    #[error(transparent)]
    IoError(#[from] IoError),
    /// An error occurred during sending messages to a pd patch.
    #[error(transparent)]
    SendError(#[from] SendError),
    /// An error occurred during subscription to senders in a pd patch.
    #[error(transparent)]
    SubscriptionError(#[from] SubscriptionError),
    /// An error occurred related to sizes of entities.
    #[error(transparent)]
    SizeError(#[from] SizeError),
    /// An error occurred related to pd arrays.
    #[error(transparent)]
    ArrayError(#[from] ArrayError),
    /// An error occurred related to pd instances.
    #[error(transparent)]
    InstanceError(#[from] InstanceError),
    /// An error occurred related to string conversion.
    ///
    /// `CString` or `CStr` conversion error.
    #[error(transparent)]
    StringConversionError(#[from] StringConversionError),
}

/// Errors related to initialization.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum InitializationError {
    // We are deprecating this error.
    /// pd could not be initialized two times.
    // #[error("Pure Data is already initialized.")]
    // AlreadyInitialized,

    /// Error in initializing ring buffers.
    #[error("Failed to initialize ring buffers which are needed for the message queue.")]
    RingBufferInitializationError,
    /// A failure happened in pd initialization with an unknown reason.
    #[error("An unknown error occurred in Pure Data initialization.")]
    InitializationFailed,
}

/// Errors related to audio initialization.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum AudioInitializationError {
    /// A failure happened in pd audio initialization with an unknown reason.
    #[error("An unknown error occurred in Pure Data audio initialization.")]
    InitializationFailed,
}

/// Errors related to a lifecycle of a pd patch.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum PatchLifeCycleError {
    /// Failed to open patch for unknown reason.
    #[error("Failed to open patch.")]
    FailedToOpenPatch,
    /// Failed to close patch, because the handle which was provided is null.
    #[error("Failed to close patch, because the handle which was provided is null.")]
    FailedToClosePatch,
    /// The string which is passed could not be evaluated as a patch.
    #[error("The string which is passed could not be evaluated as a patch.")]
    FailedToEvaluateAsPatch { content: String, msg: String },
    /// The patch which is trying to be communicated with is not open.
    #[error("The patch which is trying to be communicated with is not open.")]
    PatchIsNotOpen,
    /// The path to the patch which are being tried to open is invalid.
    #[error("The path you have provided does not exist in the file system. Path: {0}")]
    PathDoesNotExist(String),
    /// An error occurred related to string conversion.
    ///
    /// `CString` or `CStr` conversion error.
    #[error(transparent)]
    StringConversion(#[from] StringConversionError),
}

/// Errors related to a lifecycle of a pd gui.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum GuiLifeCycleError {
    /// Failed to open gui, most probably because the path is invalid to the pd binary.
    #[error("Failed to open gui, please provide a valid path to the pd binary.")]
    FailedToOpenGui,
    /// An error occurred related to string conversion.
    ///
    /// `CString` or `CStr` conversion error.
    #[error(transparent)]
    StringConversion(#[from] StringConversionError),
}

/// Errors related to general filesystem access.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum IoError {
    /// The path to the patch which are being tried to open is invalid.
    #[error("The path you have provided does not exist in the file system. Path: {0}")]
    PathDoesNotExist(String),
    /// An error occurred related to string conversion.
    ///
    /// `CString` or `CStr` conversion error.
    #[error(transparent)]
    StringConversion(#[from] StringConversionError),
}

/// Errors related to sending messages to a pd patch.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SendError {
    /// The pd patch does not contain a receiver with the name you provided.
    #[error("No destination found for receiver: `{0}` in loaded pd patch.")]
    MissingDestination(String),
    /// A general error when the values which you are sending to the receiver are out of range.
    #[error("Values which are being sent are out of range.")]
    OutOfRange,
    /// An error occurred related to string conversion.
    ///
    /// `CString` or `CStr` conversion error.
    #[error(transparent)]
    StringConversion(#[from] StringConversionError),
}

/// Errors related to subscription to senders in a pd patch.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SubscriptionError {
    /// A failure of subscription to a sender with an unknown reason.
    #[error("Failed to subscribe to sender: `{0}` in loaded pd patch.")]
    FailedToSubscribeToSender(String),
    /// An error occurred related to string conversion.
    ///
    /// `CString` or `CStr` conversion error.
    #[error(transparent)]
    StringConversion(#[from] StringConversionError),
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
    /// An error occurred related to string conversion.
    ///
    /// `CString` or `CStr` conversion error.
    #[error(transparent)]
    StringConversion(#[from] StringConversionError),
}

/// Errors related to pd arrays.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum ArrayError {
    /// The array which is being tried to be accessed doesn't exist.
    #[error("The array which you're trying to access doesn't exist.")]
    FailedToFindArray,
    /// The position in the array which is tried to be written is out of bounds.
    #[error("The position in array which you're trying to write is out of bounds.")]
    OutOfBounds,
    /// An error occurred related to string conversion.
    ///
    /// `CString` or `CStr` conversion error.
    #[error(transparent)]
    StringConversion(#[from] StringConversionError),
}

/// Errors related to pd instances.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum InstanceError {
    /// The instance which is being tried to be created already exists.
    #[error("The instance {0} already created and exists.")]
    InstanceAlreadyExists(String),
    /// The instance which is being tried to be accessed does not exist.
    #[error("The instance {0} does not exist.")]
    InstanceDoesNotExist(String),
    /// The instance which is being tried to be accessed does not exist.
    #[error("The instance failed to create. Error: {0}")]
    InstanceFailedToCreate(String),
}

/// Errors related to string conversion.
///
/// `CString` or `CStr` conversion error.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum StringConversionError {
    /// An interior null byte was found in the middle while converting to `CString` or `CStr`.
    #[error("An interior null byte was found at position {pos} while converting to CString. Original string: '{string}' (bytes: {bytes:?})",
        pos = .0,
        string = String::from_utf8_lossy(.1),
        bytes = .1
    )]
    NullTerminatedString(usize, Vec<u8>),
}

impl From<NulError> for StringConversionError {
    fn from(err: NulError) -> Self {
        Self::NullTerminatedString(err.nul_position(), err.into_vec())
    }
}
