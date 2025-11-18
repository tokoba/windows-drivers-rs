// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

use wdk_sys::{
    NTSTATUS,
    WDF_OBJECT_ATTRIBUTES,
    WDF_TIMER_CONFIG,
    WDFTIMER,
    call_unsafe_wdf_function_binding,
};

use crate::nt_success;

/// Safe wrapper around a WDF timer for scheduling deferred execution.
///
/// A WDF timer allows drivers to schedule callback functions to execute after
/// a specified time interval. Timers run at DISPATCH_LEVEL as DPCs (Deferred
/// Procedure Calls), making them suitable for time-sensitive operations that
/// don't require high-precision timing.
///
/// # Use Cases
///
/// - Implementing timeouts for I/O operations
/// - Polling hardware status at regular intervals
/// - Implementing watchdog timers
/// - Scheduling periodic maintenance tasks
/// - Implementing retry logic with delays
///
/// # Timer Types
///
/// - **One-shot**: Timer fires once after the specified interval
/// - **Periodic**: Timer fires repeatedly at regular intervals
///
/// The timer type is configured via `WDF_TIMER_CONFIG` when creating the timer.
///
/// # IRQL Requirements
///
/// - **Timer callback**: Executes at DISPATCH_LEVEL (as a DPC)
/// - **Start/Stop**: Can be called at IRQL <= DISPATCH_LEVEL
///
/// # Time Units
///
/// All time values are in 100-nanosecond units:
/// - 1 millisecond = 10,000 units
/// - 1 second = 10,000,000 units
/// - Negative values = relative time from now
/// - Positive values = absolute system time
///
/// # Example: One-Shot Timer
///
/// ```rust,no_run
/// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
/// # {
/// use wdk::wdf::Timer;
/// use wdk_sys::{WDF_TIMER_CONFIG, WDF_OBJECT_ATTRIBUTES};
///
/// let mut config = WDF_TIMER_CONFIG::default();
/// // config.EvtTimerFunc = Some(my_timer_callback);
///
/// let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
/// let timer = Timer::try_new(&mut config, &mut attrs)?;
///
/// // Fire once after 100ms (100ms * 10,000 = 1,000,000)
/// timer.start(-1_000_000);
/// # Ok::<(), wdk_sys::NTSTATUS>(())
/// # }
/// ```
///
/// # Example: Periodic Timer
///
/// ```rust,no_run
/// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
/// # {
/// use wdk::wdf::Timer;
/// use wdk_sys::{WDF_TIMER_CONFIG, WDF_OBJECT_ATTRIBUTES};
///
/// let mut config = WDF_TIMER_CONFIG::default();
/// // config.Period = 1000; // Repeat every 100ms
/// // config.EvtTimerFunc = Some(my_timer_callback);
///
/// let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
/// let timer = Timer::try_new(&mut config, &mut attrs)?;
///
/// timer.start(-1_000_000); // Start after 100ms, then repeat
/// # Ok::<(), wdk_sys::NTSTATUS>(())
/// # }
/// ```
///
/// # Lifetime and Deletion
///
/// The timer is automatically deleted when its parent WDF object is deleted.
/// Always stop the timer before allowing its parent to be deleted.
///
/// # Thread Safety
///
/// This type is `Send` and `Sync` - it can be safely shared between threads.
/// The WDF framework ensures proper synchronization of timer operations.
///
/// # See Also
///
/// - [WdfTimerCreate](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdftimer/nf-wdftimer-wdftimercreate)
/// - [WdfTimerStart](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdftimer/nf-wdftimer-wdftimers start)
/// - [WdfTimerStop](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdftimer/nf-wdftimer-wdftimerstop)
/// - [Using Timers](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/using-timers)
pub struct Timer {
    /// Handle to the underlying WDF timer object.
    ///
    /// This field is private to ensure the timer is always in a valid state
    /// and can only be manipulated through safe methods.
    wdf_timer: WDFTIMER,
}
impl Timer {
    /// Creates a new WDF timer object.
    ///
    /// # Parameters
    ///
    /// - `timer_config`: Configuration specifying timer behavior:
    ///   - `EvtTimerFunc`: Callback function to execute when timer fires
    ///   - `Period`: For periodic timers, the repeat interval (0 = one-shot)
    ///   - `TolerableDelay`: Acceptable delay for timer coalescing (power saving)
    ///   - `UseHighResolutionTimer`: Use high-resolution timer (more precise but higher overhead)
    /// - `attributes`: Object attributes for the timer:
    ///   - `ParentObject`: Parent WDF object (timer deleted when parent is deleted)
    ///   - `ExecutionLevel`: IRQL level for timer callback
    ///   - `SynchronizationScope`: Synchronization with other callbacks
    ///
    /// # Returns
    ///
    /// - `Ok(Timer)`: Successfully created timer
    /// - `Err(NTSTATUS)`: Failure status code
    ///
    /// # Errors
    ///
    /// Common error codes:
    /// - `STATUS_INSUFFICIENT_RESOURCES`: Out of memory
    /// - `STATUS_INVALID_PARAMETER`: Invalid configuration or attributes
    /// - `STATUS_WDF_PARENT_NOT_SPECIFIED`: No parent object specified
    ///
    /// Full documentation: [WdfTimerCreate Return Values](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdftimer/nf-wdftimer-wdftimercreate#return-value)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
    /// # {
    /// use wdk::wdf::Timer;
    /// use wdk_sys::{WDF_TIMER_CONFIG, WDF_OBJECT_ATTRIBUTES};
    ///
    /// let mut config = WDF_TIMER_CONFIG::default();
    /// // Configure timer callback and behavior
    ///
    /// let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
    /// // Set parent device object
    ///
    /// match Timer::try_new(&mut config, &mut attrs) {
    ///     Ok(timer) => { /* use timer */ }
    ///     Err(status) => { /* handle error */ }
    /// }
    /// # }
    /// ```
    pub fn try_new(
        timer_config: &mut WDF_TIMER_CONFIG,
        attributes: &mut WDF_OBJECT_ATTRIBUTES,
    ) -> Result<Self, NTSTATUS> {
        let mut timer = Self {
            wdf_timer: core::ptr::null_mut(),
        };

        let nt_status;
        // SAFETY: The resulting ffi object is stored in a private member and not
        // accessible outside of this module, and this module guarantees that it is
        // always in a valid state.
        unsafe {
            nt_status = call_unsafe_wdf_function_binding!(
                WdfTimerCreate,
                timer_config,
                attributes,
                &mut timer.wdf_timer as *mut WDFTIMER,
            );
        }
        nt_success(nt_status).then_some(timer).ok_or(nt_status)
    }

    /// Try to construct a WDF Timer object
    ///
    /// # Errors
    ///
    /// This function will return an error if WDF fails to construct a timer.
    /// The error variant will contain a [`NTSTATUS`] of the failure. Full error
    /// documentation is available in the [WDFTimer Documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdftimer/nf-wdftimer-wdftimercreate#return-value)
    pub fn create(
        timer_config: &mut WDF_TIMER_CONFIG,
        attributes: &mut WDF_OBJECT_ATTRIBUTES,
    ) -> Result<Self, NTSTATUS> {
        Self::try_new(timer_config, attributes)
    }

    /// Starts the timer with the specified due time.
    ///
    /// For one-shot timers, the timer fires once after the specified interval.
    /// For periodic timers, the timer fires first after `due_time`, then repeatedly
    /// at the interval specified by `WDF_TIMER_CONFIG::Period`.
    ///
    /// # Parameters
    ///
    /// - `due_time`: Time until the timer fires, in 100-nanosecond units:
    ///   - **Negative values**: Relative time from now (e.g., -10,000,000 = 1 second from now)
    ///   - **Positive values**: Absolute system time (rarely used)
    ///   - **Zero**: Fires immediately
    ///
    /// # Returns
    ///
    /// - `true`: Timer was not previously running (started successfully)
    /// - `false`: Timer was already running (restarted with new due time)
    ///
    /// # Behavior
    ///
    /// If the timer is already running, this call:
    /// - Cancels the previous timer setting
    /// - Restarts the timer with the new due time
    /// - Returns `false` to indicate it was already active
    ///
    /// # IRQL
    ///
    /// Can be called at IRQL <= DISPATCH_LEVEL.
    ///
    /// # Examples
    ///
    /// ## Fire Once After 100ms
    ///
    /// ```rust,no_run
    /// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
    /// # {
    /// # use wdk::wdf::Timer;
    /// # use wdk_sys::{WDF_TIMER_CONFIG, WDF_OBJECT_ATTRIBUTES};
    /// # let mut config = WDF_TIMER_CONFIG::default();
    /// # let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
    /// # let timer = Timer::try_new(&mut config, &mut attrs)?;
    /// // 100ms = 100 * 10,000 = 1,000,000 (100ns units)
    /// let was_running = timer.start(-1_000_000);
    /// # Ok::<(), wdk_sys::NTSTATUS>(())
    /// # }
    /// ```
    ///
    /// ## Fire Immediately
    ///
    /// ```rust,no_run
    /// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
    /// # {
    /// # use wdk::wdf::Timer;
    /// # use wdk_sys::{WDF_TIMER_CONFIG, WDF_OBJECT_ATTRIBUTES};
    /// # let mut config = WDF_TIMER_CONFIG::default();
    /// # let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
    /// # let timer = Timer::try_new(&mut config, &mut attrs)?;
    /// timer.start(0);
    /// # Ok::<(), wdk_sys::NTSTATUS>(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`stop()`](Self::stop)
    /// - [WdfTimerStart](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdftimer/nf-wdftimer-wdftimerstop)
    #[must_use]
    pub fn start(&self, due_time: i64) -> bool {
        let result;
        // SAFETY: `wdf_timer` is a private member of `Timer`, originally created by
        // WDF, and this module guarantees that it is always in a valid state.
        unsafe {
            result = call_unsafe_wdf_function_binding!(WdfTimerStart, self.wdf_timer, due_time);
        }
        result != 0
    }

    /// Stops the timer and optionally waits for any running callback to complete.
    ///
    /// This method cancels a pending timer or waits for a currently executing
    /// timer callback to finish, depending on the `wait` parameter.
    ///
    /// # Parameters
    ///
    /// - `wait`: Whether to wait for a running timer callback to complete:
    ///   - `true`: Block until the timer callback (if running) completes
    ///   - `false`: Cancel the timer but don't wait for callback completion
    ///
    /// # Returns
    ///
    /// - `true`: Timer was running and has been stopped
    /// - `false`: Timer was not running
    ///
    /// # IRQL Requirements
    ///
    /// - **`wait = true`**: Must be called at IRQL = PASSIVE_LEVEL
    ///   (Cannot wait at elevated IRQL)
    /// - **`wait = false`**: Can be called at IRQL <= DISPATCH_LEVEL
    ///
    /// # Behavior with `wait = true`
    ///
    /// When `wait` is `true`:
    /// - If timer callback is running, blocks until it completes
    /// - If timer is pending, cancels it
    /// - Guarantees callback won't run after this returns
    /// - **Deadlock risk**: Don't call from within the timer's own callback!
    ///
    /// # Behavior with `wait = false`
    ///
    /// When `wait` is `false`:
    /// - Cancels any pending timer
    /// - Returns immediately without waiting
    /// - Timer callback might still be running or about to run
    /// - Suitable for calling at elevated IRQL
    ///
    /// # Examples
    ///
    /// ## Stop and Wait (Cleanup Scenario)
    ///
    /// ```rust,no_run
    /// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
    /// # {
    /// # use wdk::wdf::Timer;
    /// # use wdk_sys::{WDF_TIMER_CONFIG, WDF_OBJECT_ATTRIBUTES};
    /// # let mut config = WDF_TIMER_CONFIG::default();
    /// # let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
    /// # let timer = Timer::try_new(&mut config, &mut attrs)?;
    /// // During driver cleanup at PASSIVE_LEVEL
    /// if timer.stop(true) {
    ///     // Timer was running and is now fully stopped
    /// }
    /// # Ok::<(), wdk_sys::NTSTATUS>(())
    /// # }
    /// ```
    ///
    /// ## Stop Without Waiting (DISPATCH_LEVEL)
    ///
    /// ```rust,no_run
    /// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
    /// # {
    /// # use wdk::wdf::Timer;
    /// # use wdk_sys::{WDF_TIMER_CONFIG, WDF_OBJECT_ATTRIBUTES};
    /// # let mut config = WDF_TIMER_CONFIG::default();
    /// # let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
    /// # let timer = Timer::try_new(&mut config, &mut attrs)?;
    /// // Called from a spinlock-protected region
    /// timer.stop(false); // Don't wait - can't wait at DISPATCH_LEVEL
    /// # Ok::<(), wdk_sys::NTSTATUS>(())
    /// # }
    /// ```
    ///
    /// # Deadlock Warning
    ///
    /// **Never** call `stop(true)` from within the timer's own callback function,
    /// as it will wait for itself to complete, causing a deadlock.
    ///
    /// # See Also
    ///
    /// - [`start()`](Self::start)
    /// - [WdfTimerStop](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdftimer/nf-wdftimer-wdftimerstop)
    #[must_use]
    pub fn stop(&self, wait: bool) -> bool {
        let result;
        // SAFETY: `wdf_timer` is a private member of `Timer`, originally created by
        // WDF, and this module guarantees that it is always in a valid state.
        unsafe {
            result =
                call_unsafe_wdf_function_binding!(WdfTimerStop, self.wdf_timer, u8::from(wait));
        }
        result != 0
    }
}
