// Copyright (c) Microsoft Corporation
// License: MIT OR Apache-2.0

use wdk_sys::{NTSTATUS, WDF_OBJECT_ATTRIBUTES, WDFSPINLOCK, call_unsafe_wdf_function_binding};

use crate::nt_success;

/// Safe wrapper around a WDF spinlock for thread synchronization.
///
/// A spinlock provides mutual exclusion for shared data accessed by code running
/// at `IRQL` <= `DISPATCH_LEVEL`. When a thread acquires the spinlock, the system
/// raises the IRQL to `DISPATCH_LEVEL` and prevents other threads from acquiring
/// the same lock. When released, the IRQL is restored to its previous level.
///
/// # Use Cases
///
/// Use spinlocks when you need to:
/// - Synchronize access to shared driver data
/// - Protect critical sections that run quickly (< 25 microseconds)
/// - Coordinate between interrupt service routines and driver code
/// - Prevent concurrent access to device registers
///
/// # IRQL Requirements
///
/// - **Acquire**: Can be called at IRQL <= DISPATCH_LEVEL
/// - **Release**: Must be called at IRQL = DISPATCH_LEVEL (set by acquire)
/// - **Critical section**: Runs at IRQL = DISPATCH_LEVEL
///
/// # Important Constraints
///
/// When holding a spinlock (at DISPATCH_LEVEL):
/// - ❌ Cannot call pageable code
/// - ❌ Cannot access pageable memory
/// - ❌ Cannot wait for synchronization objects
/// - ❌ Cannot call most WDF APIs (check documentation)
/// - ✅ Can access non-paged pool memory
/// - ✅ Can call other DISPATCH_LEVEL or higher IRQL functions
///
/// # Critical Section Guidelines
///
/// Keep critical sections as short as possible:
/// - < 25 microseconds is ideal
/// - Avoid loops that might not terminate quickly
/// - Don't perform I/O operations
/// - Don't call functions with unknown execution time
///
/// # Alternatives
///
/// Consider these alternatives when appropriate:
/// - **WDF Automatic Synchronization**: Simpler for single-device scenarios
/// - **Wait Locks**: When you can wait (IRQL <= APC_LEVEL)
/// - **Interrupt Locks**: For ISR-to-driver synchronization
/// - **DPC Objects**: To defer processing to lower IRQL
///
/// # Example: Basic Usage
///
/// ```rust,no_run
/// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
/// # {
/// use wdk::wdf::SpinLock;
/// use wdk_sys::WDF_OBJECT_ATTRIBUTES;
///
/// // Create spinlock during driver initialization
/// let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
/// let lock = SpinLock::try_new(&mut attrs)?;
///
/// // In a callback function, protect shared data:
/// lock.acquire();
/// // Critical section - keep this fast!
/// // Update shared state here
/// lock.release();
/// # Ok::<(), wdk_sys::NTSTATUS>(())
/// # }
/// ```
///
/// # Example: Protecting Device Context
///
/// ```rust,no_run
/// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
/// # {
/// use wdk::wdf::SpinLock;
/// use wdk_sys::WDF_OBJECT_ATTRIBUTES;
///
/// struct DeviceContext {
///     counter: u32,
///     lock: SpinLock,
/// }
///
/// fn increment_counter(ctx: &DeviceContext) {
///     ctx.lock.acquire();
///     // Safe to access ctx.counter while lock is held
///     // (pseudo-code - actual implementation needs unsafe)
///     ctx.lock.release();
/// }
/// # }
/// ```
///
/// # Lifetime and Deletion
///
/// The spinlock is automatically deleted when its parent WDF object is deleted.
/// Set the parent using `WDF_OBJECT_ATTRIBUTES::ParentObject` when creating.
///
/// # Thread Safety
///
/// This type is `Send` and `Sync` - it can be safely shared between threads.
/// The WDF framework ensures proper synchronization of the underlying handle.
///
/// # See Also
///
/// - [WdfSpinLockCreate](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfsync/nf-wdfsync-wdfspinlockcreate)
/// - [WdfSpinLockAcquire](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfsync/nf-wdfsync-wdfspinlockacquire)
/// - [WdfSpinLockRelease](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfsync/nf-wdfsync-wdfspinlockrelease)
/// - [Using Framework Locks](https://learn.microsoft.com/en-us/windows-hardware/drivers/wdf/using-framework-locks)
pub struct SpinLock {
    /// Handle to the underlying WDF spinlock object.
    ///
    /// This field is private to ensure the spinlock is always in a valid state
    /// and can only be manipulated through safe methods.
    wdf_spin_lock: WDFSPINLOCK,
}
impl SpinLock {
    /// Creates a new WDF spinlock object.
    ///
    /// # Parameters
    ///
    /// - `attributes`: Object attributes that control the spinlock's properties:
    ///   - `ParentObject`: Parent WDF object (spinlock deleted when parent is deleted)
    ///   - `ExecutionLevel`: Execution level for callbacks (not applicable to spinlocks)
    ///   - `SynchronizationScope`: Synchronization scope (not applicable to spinlocks)
    ///
    /// # Returns
    ///
    /// - `Ok(SpinLock)`: Successfully created spinlock
    /// - `Err(NTSTATUS)`: Failure status code (typically `STATUS_INSUFFICIENT_RESOURCES`)
    ///
    /// # Errors
    ///
    /// Common error codes:
    /// - `STATUS_INSUFFICIENT_RESOURCES`: Out of memory
    /// - `STATUS_INVALID_PARAMETER`: Invalid attributes
    ///
    /// Full error documentation: [WdfSpinLockCreate Return Values](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfsync/nf-wdfsync-wdfspinlockcreate#return-value)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
    /// # {
    /// use wdk::wdf::SpinLock;
    /// use wdk_sys::WDF_OBJECT_ATTRIBUTES;
    ///
    /// let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
    /// // attrs.ParentObject = device; // Set parent for automatic cleanup
    ///
    /// match SpinLock::try_new(&mut attrs) {
    ///     Ok(lock) => { /* use lock */ }
    ///     Err(status) => { /* handle error */ }
    /// }
    /// # }
    /// ```
    pub fn try_new(attributes: &mut WDF_OBJECT_ATTRIBUTES) -> Result<Self, NTSTATUS> {
        let mut spin_lock = Self {
            wdf_spin_lock: core::ptr::null_mut(),
        };

        let nt_status;
        // SAFETY: The resulting ffi object is stored in a private member and not
        // accessible outside of this module, and this module guarantees that it is
        // always in a valid state.
        unsafe {
            nt_status = call_unsafe_wdf_function_binding!(
                WdfSpinLockCreate,
                attributes,
                &mut spin_lock.wdf_spin_lock as *mut _,
            );
        }
        nt_success(nt_status).then_some(spin_lock).ok_or(nt_status)
    }

    /// Try to construct a WDF Spin Lock object. This is an alias for
    /// [`SpinLock::try_new()`]
    ///
    /// # Errors
    ///
    /// This function will return an error if WDF fails to construct a spinlock.
    /// The error variant will contain a [`NTSTATUS`] of the failure. Full error
    /// documentation is available in the [WDFSpinLock Documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfsync/nf-wdfsync-wdfspinlockcreate#return-value)
    pub fn create(attributes: &mut WDF_OBJECT_ATTRIBUTES) -> Result<Self, NTSTATUS> {
        Self::try_new(attributes)
    }

    /// Acquires the spinlock, raising IRQL to DISPATCH_LEVEL.
    ///
    /// This method blocks the current thread until the spinlock is acquired.
    /// Upon acquisition, the system raises the thread's IRQL to DISPATCH_LEVEL,
    /// preventing preemption and ensuring exclusive access.
    ///
    /// # Behavior
    ///
    /// - **Blocking**: Spins (busy-waits) until the lock is available
    /// - **IRQL change**: Raises from current IRQL to DISPATCH_LEVEL
    /// - **Non-reentrant**: Acquiring the same lock twice causes deadlock
    ///
    /// # IRQL
    ///
    /// - **Called at**: IRQL <= DISPATCH_LEVEL
    /// - **Returns at**: IRQL = DISPATCH_LEVEL
    ///
    /// # Constraints After Acquisition
    ///
    /// After calling `acquire()`, until `release()` is called:
    /// - Cannot access pageable code or data
    /// - Cannot call functions that wait
    /// - Cannot call most WDF APIs
    /// - Keep the critical section as short as possible
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
    /// # {
    /// # use wdk::wdf::SpinLock;
    /// # use wdk_sys::WDF_OBJECT_ATTRIBUTES;
    /// # let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
    /// # let lock = SpinLock::try_new(&mut attrs)?;
    /// lock.acquire();
    /// // Critical section at DISPATCH_LEVEL
    /// // Access shared data here
    /// lock.release(); // Always release!
    /// # Ok::<(), wdk_sys::NTSTATUS>(())
    /// # }
    /// ```
    ///
    /// # Deadlock Warning
    ///
    /// Always ensure:
    /// - Every `acquire()` is matched with a `release()`
    /// - Don't acquire the same lock twice from the same thread
    /// - Use consistent lock ordering when acquiring multiple locks
    ///
    /// # See Also
    ///
    /// - [`release()`](Self::release)
    /// - [WdfSpinLockAcquire](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfsync/nf-wdfsync-wdfspinlockacquire)
    pub fn acquire(&self) {
        // SAFETY: `wdf_spin_lock` is a private member of `SpinLock`, originally created
        // by WDF, and this module guarantees that it is always in a valid state.
        unsafe {
            call_unsafe_wdf_function_binding!(WdfSpinLockAcquire, self.wdf_spin_lock);
        }
    }

    /// Releases the spinlock, restoring the previous IRQL.
    ///
    /// This method releases the spinlock and lowers the thread's IRQL back to
    /// the level it was at before [`acquire()`](Self::acquire) was called.
    ///
    /// # IRQL
    ///
    /// - **Called at**: IRQL = DISPATCH_LEVEL (must have called `acquire()` first)
    /// - **Returns at**: IRQL = previous level (before `acquire()`)
    ///
    /// # Requirements
    ///
    /// - Must be called after `acquire()` on the same thread
    /// - Must be called at DISPATCH_LEVEL (set by `acquire()`)
    /// - Must be called in the same function or scope as `acquire()` (best practice)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(any(driver_model__driver_type = "KMDF", driver_model__driver_type = "UMDF"))]
    /// # {
    /// # use wdk::wdf::SpinLock;
    /// # use wdk_sys::WDF_OBJECT_ATTRIBUTES;
    /// # let mut attrs = WDF_OBJECT_ATTRIBUTES::default();
    /// # let lock = SpinLock::try_new(&mut attrs)?;
    /// lock.acquire();
    /// // ... critical section ...
    /// lock.release(); // Restores previous IRQL
    /// # Ok::<(), wdk_sys::NTSTATUS>(())
    /// # }
    /// ```
    ///
    /// # Safety Note
    ///
    /// Calling `release()` without first calling `acquire()` is undefined behavior
    /// and will likely cause a system crash. Always pair acquire/release calls.
    ///
    /// # See Also
    ///
    /// - [`acquire()`](Self::acquire)
    /// - [WdfSpinLockRelease](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdfsync/nf-wdfsync-wdfspinlockrelease)
    pub fn release(&self) {
        // SAFETY: `wdf_spin_lock` is a private member of `SpinLock`, originally created
        // by WDF, and this module guarantees that it is always in a valid state.
        unsafe {
            call_unsafe_wdf_function_binding!(WdfSpinLockRelease, self.wdf_spin_lock);
        }
    }
}
