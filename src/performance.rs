//! Profiling and performance monitoring tools
#[cfg(feature = "timers")]
use web_sys::console;

pub(crate) struct Timer<'a> {
    #[allow(dead_code)]
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        #[cfg(feature = "timers")]
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        #[cfg(feature = "timers")]
        console::time_end_with_label(self.name);
    }
}
