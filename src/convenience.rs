#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

use crate::{
    error::{InitializationError, LibpdError, PatchLifeCycleError},
    types::{PatchFileHandle, ReceiverHandle},
};

/// Activates audio in pd.
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`SendError`](crate::error::SendError)
///   - [`MissingDestination`](crate::error::SendError::MissingDestination)
/// - [`SizeError`](crate::error::SizeError)
///   - [`TooLarge`](crate::error::SizeError::TooLarge)
///
/// To match over these errors, you would need to downcast the returned error.
pub fn dsp_on() -> Result<(), Box<dyn LibpdError>> {
    crate::send::start_message(1)?;
    crate::send::add_float_to_started_message(1.0);
    crate::send::finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}

/// De-activates audio in pd.
///
/// # Errors
///
/// A list of errors that can occur:
/// - [`SendError`](crate::error::SendError)
///   - [`MissingDestination`](crate::error::SendError::MissingDestination)
/// - [`SizeError`](crate::error::SizeError)
///   - [`TooLarge`](crate::error::SizeError::TooLarge)
///
/// To match over these errors, you would need to downcast the returned error.
pub fn dsp_off() -> Result<(), Box<dyn LibpdError>> {
    crate::send::start_message(1)?;
    crate::send::add_float_to_started_message(0.0);
    crate::send::finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}

/// Find the number of pd ticks according to the case.
///
/// The calculation is `buffer_size / (block_size * channels)`
#[must_use]
#[allow(clippy::integer_division)]
pub fn calculate_ticks(channels: i32, buffer_size: i32) -> i32 {
    let block_size = crate::block_size();
    buffer_size / (block_size * channels)
}

/// An abstraction provided for convenience to track the state of pd and execute some common functions.
///
/// Pd initializes globally.
///
/// This is one of the reasons that there are more bare functions in this crate than data structures and abstractions.
/// This has some advantages and disadvantages. In the case of [`PdGlobal`] we can not fully trust the state we track here.
/// To trust it we need to not mix the bare functions which [`PdGlobal`] wraps and member functions of [`PdGlobal`] together.
///
/// # Example of an unwanted mix
///
/// ```rust
/// use libpd_rs::convenience::PdGlobal;
/// use libpd_rs::convenience::dsp_off;
///
/// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
///
/// // We call the member function of [`PdGlobal`] to activate audio
/// // which calls [`dsp_on`] internally which then sends a message
/// // to globally initialized pd to activate dsp.
/// pd.activate_audio(true).unwrap();
///
/// // So far so good.
/// assert_eq!(pd.audio_active(), true);
///
/// // But we can send messages to globally initialized pd many ways
/// // and here is one of the ways we can do it.
/// dsp_off().unwrap();
///
/// // But now [`PdGlobal`] is not aware of the state
/// // of the globally initialized pd in the background.
/// // The information it holds is outdated and not true anymore.
/// assert_eq!(pd.audio_active(), true);
/// ```
///
/// To avoid this situation if you use [`PdGlobal`] check its member functions and only use them and **not** their bare counterparts.
///
/// There are many bare functions in this crate which is not wrapped by [`PdGlobal`] and those are safe to use while using [`PdGlobal`] related functions.
pub struct PdGlobal {
    audio_active: bool,
    input_channels: i32,
    output_channels: i32,
    sample_rate: i32,
    running_patch: Option<PatchFileHandle>,
    temporary_evaluated_patch: Option<NamedTempFile>,
    /// A store to keep track of subscriptions which are made to senders in pd through the app lifecycle.
    pub subscriptions: HashMap<String, ReceiverHandle>,
    /// A store to keep track of paths which are added to pd search paths through the app lifecycle.
    pub search_paths: Vec<PathBuf>,
}

impl PdGlobal {
    /// Initializes pd globally.
    ///
    /// It calls [`init`](crate::init) and [`initialize_audio`](crate::initialize_audio) with the provided arguments and returns an instance of [`PdGlobal`] where a user can keep simple state and call some convenience methods.
    /// It would be wise to call this function before anything pd related.
    ///
    /// Please use only one instance of this struct, because of how [`libpd`](https://github.com/libpd/libpd) is designed the underlying initialization is scoped globally.
    ///
    /// # Examples
    /// ```rust
    /// use libpd_rs::convenience::PdGlobal;
    ///
    /// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`InitializationError`](crate::error::InitializationError)
    ///   - [`RingBufferInitializationError`](crate::error::InitializationError::RingBufferInitializationError)
    ///   - [`InitializationFailed`](crate::error::InitializationError::InitializationFailed)
    /// - [`AudioInitializationError`](crate::error::AudioInitializationError)
    ///   - [`AudioInitializationFailed`](crate::error::AudioInitializationError::AudioInitializationFailed)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn init_and_configure(
        input_channels: i32,
        output_channels: i32,
        sample_rate: i32,
    ) -> Result<Self, Box<dyn LibpdError>> {
        match crate::init() {
            Ok(_) => (),
            Err(err) => match err {
                // Ignore re-initialization errors.
                InitializationError::AlreadyInitialized => (),
                err => return Err(err.into()),
            },
        }
        crate::initialize_audio(input_channels, output_channels, sample_rate)?;
        Ok(Self {
            audio_active: false,
            input_channels,
            output_channels,
            sample_rate,
            running_patch: None,
            temporary_evaluated_patch: None,
            subscriptions: HashMap::default(),
            search_paths: vec![],
        })
    }

    /// Adds a path to the list of paths where libpd searches in.
    ///
    /// Relative paths are relative to the current working directory.
    /// Unlike the desktop pd application, **no** search paths are set by default.
    pub fn add_path_to_search_paths<T: AsRef<Path>>(
        &mut self,
        path: T,
    ) -> Result<(), Box<dyn LibpdError>> {
        let path = path.as_ref().to_path_buf();
        if !self.search_paths.contains(&path) {
            crate::add_to_search_paths(path.clone())?;
            self.search_paths.push(path);
        }
        Ok(())
    }

    /// Adds many paths to the list of paths where libpd searches in.
    ///
    /// Relative paths are relative to the current working directory.
    /// Unlike the desktop pd application, **no** search paths are set by default.
    pub fn add_paths_to_search_paths<T: AsRef<Path>>(
        &mut self,
        paths: &[T],
    ) -> Result<(), Box<dyn LibpdError>> {
        for path in paths {
            if !self.search_paths.contains(&path.as_ref().to_path_buf()) {
                crate::add_to_search_paths(path)?;
                self.search_paths.push(path.as_ref().to_path_buf());
            }
        }
        Ok(())
    }

    /// Clears all the paths where libpd searches for patches and assets.
    pub fn clear_all_search_paths(&mut self) {
        crate::clear_search_paths();
        self.search_paths.clear();
    }

    /// Closes a pd patch.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`PatchLifeCycleError`](crate::error::PatchLifeCycleError)
    ///   - [`FailedToClosePatch`](crate::error::PatchLifeCycleError::FailedToClosePatch)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn close_patch(&mut self) -> Result<(), Box<dyn LibpdError>> {
        if let Some(handle) = self.running_patch.take() {
            crate::close_patch(handle)?;
        }
        self.temporary_evaluated_patch.take();
        Ok(())
    }

    /// Opens a pd patch.
    ///
    /// The argument should be an absolute path to the patch file.
    /// Absolute and relative paths are supported.
    /// Relative paths and single file names are tried in executable directory and manifest directory.
    ///
    /// Tha function **first** checks the executable directory and **then** the manifest directory.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::convenience::PdGlobal;
    ///
    /// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
    /// assert!(pd.open_patch("tests/patches/sine.pd").is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`PatchLifeCycleError`](crate::error::PatchLifeCycleError)
    ///   - [`FailedToClosePatch`](crate::error::PatchLifeCycleError::FailedToClosePatch)
    ///   - [`FailedToOpenPatch`](crate::error::PatchLifeCycleError::FailedToOpenPatch)
    ///   - [`PathDoesNotExist`](crate::error::PatchLifeCycleError::PathDoesNotExist)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn open_patch<T: AsRef<Path>>(&mut self, path: T) -> Result<(), Box<dyn LibpdError>> {
        if self.running_patch.is_some() {
            self.close_patch()?;
        }
        self.running_patch = Some(crate::open_patch(path)?);
        Ok(())
    }

    /// Evaluate a string as a pd patch.
    ///
    /// This function creates a temporary file with the contents passed behind the scenes.
    /// and saves it into the [`PdGlobal`] struct holding onto it until the patch is closed or the instantiated [`PdGlobal`] is dropped.
    ///
    /// Note: The patch opened after this evaluation could be closed safely with [`close_patch`](PdGlobal::close_patch).
    ///
    /// # Examples
    /// ```rust
    /// use libpd_rs::convenience::PdGlobal;
    ///
    /// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
    ///     
    /// assert!(pd.eval_patch(
    /// r#"
    /// #N canvas 577 549 158 168 12;
    /// #X obj 23 116 dac~;
    /// #X obj 23 17 osc~ 440;
    /// #X obj 23 66 *~ 0.1;
    /// #X obj 81 67 *~ 0.1;
    /// #X connect 1 0 2 0;
    /// #X connect 1 0 3 0;
    /// #X connect 2 0 0 0;
    /// #X connect 3 0 0 1;
    /// "#
    /// ,).is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`PatchLifeCycleError`](crate::error::PatchLifeCycleError)
    ///   - [`FailedToEvaluateAsPatch`](crate::error::PatchLifeCycleError::FailedToEvaluateAsPatch)
    ///   - [`FailedToClosePatch`](crate::error::PatchLifeCycleError::FailedToClosePatch)
    ///   - [`FailedToOpenPatch`](crate::error::PatchLifeCycleError::FailedToOpenPatch)
    ///   - [`PathDoesNotExist`](crate::error::PatchLifeCycleError::PathDoesNotExist)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn eval_patch<T: AsRef<str>>(&mut self, contents: T) -> Result<(), Box<dyn LibpdError>> {
        if self.running_patch.is_some() {
            self.close_patch()?;
        }
        let temp_file =
            NamedTempFile::new().map_err(|err| PatchLifeCycleError::FailedToEvaluateAsPatch {
                content: contents.as_ref().to_owned(),
                msg: err.to_string(),
            })?;
        std::fs::write(temp_file.path(), contents.as_ref()).map_err(|err| {
            PatchLifeCycleError::FailedToEvaluateAsPatch {
                content: contents.as_ref().to_owned(),
                msg: err.to_string(),
            }
        })?;
        self.running_patch = Some(crate::open_patch(temp_file.path())?);
        self.temporary_evaluated_patch = Some(temp_file);
        Ok(())
    }

    /// Starts listening messages from a source.
    ///
    /// If the source is already being listened to, this function will early return not doing anything without an error.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::convenience::PdGlobal;
    ///
    /// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to("sender").unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`SubscriptionError`](crate::error::SubscriptionError)
    ///   - [`FailedToSubscribeToSender`](crate::error::SubscriptionError::FailedToSubscribeToSender)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn subscribe_to<T: AsRef<str>>(&mut self, source: T) -> Result<(), Box<dyn LibpdError>> {
        if self.subscriptions.contains_key(source.as_ref()) {
            return Ok(());
        }
        self.subscriptions.insert(
            source.as_ref().to_owned(),
            crate::receive::start_listening_from(source.as_ref())?,
        );
        Ok(())
    }

    /// Starts listening messages from many source.
    ///
    /// If the any source is already being listened to, this function will will ignore them.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::convenience::PdGlobal;
    ///
    /// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to_many(&["sender", "other_sender"]).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`SubscriptionError`](crate::error::SubscriptionError)
    ///   - [`FailedToSubscribeToSender`](crate::error::SubscriptionError::FailedToSubscribeToSender)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn subscribe_to_many<T: AsRef<str>>(
        &mut self,
        sources: &[T],
    ) -> Result<(), Box<dyn LibpdError>> {
        for source in sources {
            if self.subscriptions.contains_key(source.as_ref()) {
                continue;
            }
            self.subscriptions.insert(
                source.as_ref().to_owned(),
                crate::receive::start_listening_from(source.as_ref())?,
            );
        }
        Ok(())
    }

    /// Stops listening messages from a source.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::convenience::PdGlobal;
    ///
    /// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to("sender").unwrap();
    /// pd.unsubscribe_from("sender");
    /// ```
    pub fn unsubscribe_from<T: AsRef<str>>(&mut self, source: T) {
        if let Some(handle) = self.subscriptions.remove(source.as_ref()) {
            crate::receive::stop_listening_from(handle);
        }
    }

    /// Stops listening messages from many sources.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::convenience::PdGlobal;
    ///
    /// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to_many(&["sender", "other_sender"]).unwrap();
    ///
    /// pd.unsubscribe_from_many(&["sender", "other_sender"]);
    /// ```
    pub fn unsubscribe_from_many<T: AsRef<str>>(&mut self, sources: &[T]) {
        for source in sources {
            if let Some(handle) = self.subscriptions.remove(source.as_ref()) {
                crate::receive::stop_listening_from(handle);
            }
        }
    }

    /// Stops listening from all sources.
    ///
    /// # Examples
    /// ```no_run
    /// use libpd_rs::convenience::PdGlobal;
    ///
    /// let pd = PdGlobal::init_and_configure(1, 2, 44100).unwrap();
    /// pd.open_patch("tests/patches/sine.pd").unwrap();
    /// pd.subscribe_to_many(&["sender", "other_sender"]).unwrap();
    ///
    /// pd.unsubscribe_from_all();
    /// ```
    pub fn unsubscribe_from_all(&mut self) {
        let sources: Vec<String> = self.subscriptions.keys().cloned().collect();
        for source in &sources {
            if let Some(handle) = self.subscriptions.remove(source) {
                crate::receive::stop_listening_from(handle);
            }
        }
    }

    /// Gets the `$0` of the running patch.
    ///
    /// `$0` id in pd could be thought as a auto generated unique identifier for the patch.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`PatchLifeCycleError`](crate::error::PatchLifeCycleError)
    ///   - [`PatchIsNotOpen`](crate::error::PatchLifeCycleError::PatchIsNotOpen)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn dollar_zero(&self) -> Result<i32, Box<dyn LibpdError>> {
        if let Some(ref patch) = self.running_patch {
            let dollar_zero = crate::get_dollar_zero(patch)?;
            return Ok(dollar_zero);
        }
        Err(Box::new(PatchLifeCycleError::PatchIsNotOpen))
    }

    /// Checks if the audio is active.
    ///
    /// The state is tracked by [`PdGlobal`].
    ///
    /// If messages sent to pd previously to activate or de-activate audio not using methods provided in this struct.
    /// This state might be false.
    #[must_use]
    pub const fn audio_active(&self) -> bool {
        self.audio_active
    }

    /// Activates or deactivates audio in pd.
    ///
    /// # Errors
    ///
    /// A list of errors that can occur:
    /// - [`SendError`](crate::error::SendError)
    ///   - [`MissingDestination`](crate::error::SendError::MissingDestination)
    /// - [`SizeError`](crate::error::SizeError)
    ///   - [`TooLarge`](crate::error::SizeError::TooLarge)
    ///
    /// To match over these errors, you would need to downcast the returned error.
    pub fn activate_audio(&mut self, on: bool) -> Result<(), Box<dyn LibpdError>> {
        if on && !self.audio_active {
            dsp_on()?;
            self.audio_active = true;
        } else if !on && self.audio_active {
            dsp_off()?;
            self.audio_active = false;
        } else {
            return Ok(());
        }
        Ok(())
    }

    /// Gets the sample rate which pd is configured with.
    ///
    /// The state is tracked by [`PdGlobal`].
    ///
    /// If anything else changes the internal state of pd it will not be reflected in this struct.
    #[must_use]
    pub const fn sample_rate(&self) -> i32 {
        self.sample_rate
    }

    /// Gets the number of input channels which pd is configured with.
    ///
    /// The state is tracked by [`PdGlobal`].
    ///
    /// If anything else changes the internal state of pd it will not be reflected in this struct.
    #[must_use]
    pub const fn input_channels(&self) -> i32 {
        self.input_channels
    }

    /// Gets the number of output channels which pd is configured with.
    ///
    /// The state is tracked by [`PdGlobal`].
    ///
    /// If anything else changes the internal state of pd it will not be reflected in this struct.
    #[must_use]
    pub const fn output_channels(&self) -> i32 {
        self.output_channels
    }
}
