#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkState {
    Available,
    NotAvailable,
}

#[cfg(target_os = "macos")]
mod macos {
    use super::NetworkState;
    use core_foundation::base::TCFType;
    use core_foundation::runloop::CFRunLoop;
    use core_foundation::string::CFString;
    use core_foundation_sys::dictionary::CFDictionaryRef;
    use core_foundation_sys::notification_center::{
        CFNotificationCenterAddObserver, CFNotificationCenterGetDistributedCenter,
        CFNotificationCenterRef, CFNotificationCenterRemoveObserver, CFNotificationName,
        CFNotificationSuspensionBehaviorDeliverImmediately,
    };
    use core_foundation_sys::runloop::kCFRunLoopDefaultMode;
    use std::ffi::c_void;
    use std::ptr;
    use tokio::sync::mpsc::error::TryRecvError;
    use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

    struct NotificationContext {
        sender: UnboundedSender<NetworkState>,
        state: NetworkState,
    }

    struct NotificationObserver {
        center: CFNotificationCenterRef,
        name: CFString,
        context: Box<NotificationContext>,
    }

    impl NotificationObserver {
        fn new(
            center: CFNotificationCenterRef,
            sender: &UnboundedSender<NetworkState>,
            state: NetworkState,
            name: &'static str,
        ) -> Self {
            let name = CFString::from_static_string(name);
            let context = Box::new(NotificationContext {
                sender: sender.clone(),
                state,
            });
            let observer = context.as_ref() as *const NotificationContext as *const c_void;

            unsafe {
                CFNotificationCenterAddObserver(
                    center,
                    observer,
                    notification_callback,
                    name.as_concrete_TypeRef(),
                    ptr::null(),
                    CFNotificationSuspensionBehaviorDeliverImmediately,
                );
            }

            Self {
                center,
                name,
                context,
            }
        }
    }

    impl Drop for NotificationObserver {
        fn drop(&mut self) {
            let observer = self.context.as_ref() as *const NotificationContext as *const c_void;
            unsafe {
                CFNotificationCenterRemoveObserver(
                    self.center,
                    observer,
                    self.name.as_concrete_TypeRef(),
                    ptr::null(),
                );
            }
        }
    }

    extern "C" fn notification_callback(
        _center: CFNotificationCenterRef,
        observer: *mut c_void,
        _name: CFNotificationName,
        _object: *const c_void,
        _user_info: CFDictionaryRef,
    ) {
        let Some(context) = (unsafe { observer.cast::<NotificationContext>().as_ref() }) else {
            return;
        };
        let _ = context.sender.send(context.state);
    }

    pub struct NetworkNotifications {
        receiver: UnboundedReceiver<NetworkState>,
        _available: NotificationObserver,
        _not_available: NotificationObserver,
    }

    impl NetworkNotifications {
        pub fn new() -> Self {
            let center = unsafe { CFNotificationCenterGetDistributedCenter() };
            let (sender, receiver) = unbounded_channel();

            Self {
                receiver,
                _available: NotificationObserver::new(
                    center,
                    &sender,
                    NetworkState::Available,
                    "com.apple.KerberosPlugin.InternalNetworkAvailable",
                ),
                _not_available: NotificationObserver::new(
                    center,
                    &sender,
                    NetworkState::NotAvailable,
                    "com.apple.KerberosPlugin.InternalNetworkNotAvailable",
                ),
            }
        }

        pub async fn recv(&mut self) -> Option<NetworkState> {
            loop {
                match self.receiver.try_recv() {
                    Ok(state) => return Some(state),
                    Err(TryRecvError::Disconnected) => return None,
                    Err(TryRecvError::Empty) => {}
                }

                CFRunLoop::run_in_mode(
                    unsafe { kCFRunLoopDefaultMode },
                    std::time::Duration::from_millis(100),
                    false,
                );
                tokio::task::yield_now().await;
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos::NetworkNotifications;

#[cfg(not(target_os = "macos"))]
pub struct NetworkNotifications;

#[cfg(not(target_os = "macos"))]
impl NetworkNotifications {
    pub fn new() -> Self {
        Self
    }

    pub async fn recv(&mut self) -> Option<NetworkState> {
        std::future::pending().await
    }
}
