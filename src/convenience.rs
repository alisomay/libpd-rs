#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, tempfile, NamedTempFile};

use crate::{
    error::{InitializationError, LibpdError, PatchLifeCycleError},
    types::{PatchFileHandle, ReceiverHandle},
};

/// Activates audio in pd.
pub fn dsp_on() -> Result<(), Box<dyn LibpdError>> {
    crate::send::start_message(1)?;
    crate::send::add_float_to_started_message(1.0);
    crate::send::finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}

/// De-activates audio in pd.
pub fn dsp_off() -> Result<(), Box<dyn LibpdError>> {
    crate::send::start_message(1)?;
    crate::send::add_float_to_started_message(0.0);
    crate::send::finish_message_as_typed_message_and_send_to("pd", "dsp")?;
    Ok(())
}

/// Find the number of pd ticks according to the case.
#[must_use]
#[allow(clippy::integer_division)]
pub fn calculate_ticks(channels: i32, buffer_size: i32) -> i32 {
    let block_size = crate::block_size();
    buffer_size / (block_size * channels)
}

// TODO:
/// Explain the role of this struct.
pub struct PdGlobal {
    /// Explain
    audio_active: bool,
    /// Explain
    input_channels: i32,
    /// Explain
    output_channels: i32,
    /// Explain
    sample_rate: i32,
    /// Explain
    running_patch: Option<PatchFileHandle>,
    /// Explain
    temporary_evaluated_patch: Option<NamedTempFile>,
    /// Explain
    pub subscriptions: HashMap<String, ReceiverHandle>,
    /// Explain
    pub search_paths: Vec<PathBuf>,
}

impl PdGlobal {
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
    /// Note: The patch opened after this evaluation could be closed safely with [`close_patch`].
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
    /// If the source is already being listened to, this function will early return but not error.
    ///
    /// # Errors
    ///
    /// Errors if the subscription fails.
    ///
    /// Which may be down casted to [`SubscriptionError::FailedToSubscribeToSender`]
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
    /// # Errors
    ///
    /// Errors if any of the subscriptions fail.
    ///
    /// Which may be down casted to [`SubscriptionError::FailedToSubscribeToSender`]
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
    pub fn unsubscribe_from<T: AsRef<str>>(&mut self, source: T) {
        if let Some(handle) = self.subscriptions.remove(source.as_ref()) {
            crate::receive::stop_listening_from(handle);
        }
    }

    /// Stops listening messages from many sources.
    pub fn unsubscribe_from_many<T: AsRef<str>>(&mut self, sources: &[T]) {
        for source in sources {
            if let Some(handle) = self.subscriptions.remove(source.as_ref()) {
                crate::receive::stop_listening_from(handle);
            }
        }
    }

    /// Stops listening from all sources.
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
    /// The returned error could be down casted to [`PatchLifeCycleError::PatchIsNotOpen`]
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

    /// Explain
    #[must_use]
    pub const fn sample_rate(&self) -> i32 {
        self.sample_rate
    }

    /// Explain
    #[must_use]
    pub const fn input_channels(&self) -> i32 {
        self.input_channels
    }

    /// Explain
    #[must_use]
    pub const fn output_channels(&self) -> i32 {
        self.output_channels
    }
}
