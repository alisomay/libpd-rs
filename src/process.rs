/// Processes the audio buffer of `f32` in place through the loaded pd patch.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::process::process_float;
/// use libpd_rs::block_size;
///
/// let output_channels = 2;
/// // ...
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0.0_f32; 512];
/// let mut output_buffer = [0.0_f32; 1024];
///
/// let buffer_size = output_buffer.len() as i32;
/// let pd_ticks: i32 = buffer_size / (block_size() * output_channels);
///
/// process_float(pd_ticks, &input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_float(pd_ticks, &[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.
pub fn process_float(ticks: i32, input_buffer: &[f32], output_buffer: &mut [f32]) {
    unsafe {
        libpd_sys::libpd_process_float(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Processes the audio buffer of `i16` in place through the loaded pd patch.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Float samples are converted to short by multiplying by `32767` and casting,
/// so any values received from pd patches beyond `-1` to `1` will result in garbage.
///
/// Note: for efficiency, does *not* clip input
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::process::process_short;
/// use libpd_rs::block_size;
///
/// let output_channels = 2;
/// // ...
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0_i16; 512];
/// let mut output_buffer = [0_i16; 1024];
///
/// let buffer_size = output_buffer.len() as i32;
/// let pd_ticks: i32 = buffer_size / (block_size() * output_channels);
///
/// process_short(pd_ticks, &input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_short(pd_ticks, &[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.

pub fn process_short(ticks: i32, input_buffer: &[i16], output_buffer: &mut [i16]) {
    unsafe {
        libpd_sys::libpd_process_short(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Processes the audio buffer of `f64` in place through the loaded pd patch.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::process::process_double;
/// use libpd_rs::block_size;
///
/// let output_channels = 2;
/// // ...
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0.0_f64; 512];
/// let mut output_buffer = [0.0_f64; 1024];
///
/// let buffer_size = output_buffer.len() as i32;
/// let pd_ticks: i32 = buffer_size / (block_size() * output_channels);
///
/// process_double(pd_ticks, &input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_double(pd_ticks, &[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.
pub fn process_double(ticks: i32, input_buffer: &[f64], output_buffer: &mut [f64]) {
    unsafe {
        libpd_sys::libpd_process_double(ticks, input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Processes the **non-interleaved** `f32` audio buffer in place through the loaded pd patch.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Copies buffer contents to/from libpd without striping.
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::process::process_raw;
///
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0.0_f32; 512];
/// let mut output_buffer = [0.0_f32; 1024];
///
/// process_raw(&input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_raw(&[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.
pub fn process_raw(input_buffer: &[f32], output_buffer: &mut [f32]) {
    unsafe {
        libpd_sys::libpd_process_raw(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Processes the **non-interleaved** `i16` audio buffer in place through the loaded pd patch.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Copies buffer contents to/from libpd without striping.
///
/// Float samples are converted to short by multiplying by `32767` and casting,
/// so any values received from pd patches beyond `-1` to `1` will result in garbage.
///
/// Note: for efficiency, does *not* clip input
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::process::process_raw_short;
///
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0_i16; 512];
/// let mut output_buffer = [0_i16; 1024];
///
/// process_raw_short(&input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_raw_short(&[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.

pub fn process_raw_short(input_buffer: &[i16], output_buffer: &mut [i16]) {
    unsafe {
        libpd_sys::libpd_process_raw_short(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}

/// Processes the **non-interleaved** `f64` audio buffer in place through the loaded pd patch.
///
/// The processing order is like the following, `input_buffer -> libpd -> output_buffer`.
///
/// Copies buffer contents to/from libpd without striping.
///
/// Call this in your **audio callback**.
///
/// # Examples
/// ```no_run
/// use libpd_rs::process::process_raw_double;
///
/// // After initializing audio and opening a patch file then in the audio callback..
///
/// // We can imagine that these are the buffers which has been handed to us by the audio callback.
/// let input_buffer = [0.0_f64; 512];
/// let mut output_buffer = [0.0_f64; 1024];
///
/// process_raw_double(&input_buffer, &mut output_buffer);
/// // Or if you wish,
/// // the input buffer can also be an empty slice if you're not going to use any inputs.
/// process_raw_double(&[], &mut output_buffer);
/// ```
/// # Panics
///
/// This function may panic for multiple reasons,
/// first of all there is a mutex lock used internally and also it processes buffers in place so there are possibilities of segfaults.
/// Use with care.
pub fn process_raw_double(input_buffer: &[f64], output_buffer: &mut [f64]) {
    unsafe {
        libpd_sys::libpd_process_raw_double(input_buffer.as_ptr(), output_buffer.as_mut_ptr());
    }
}
