#![doc = include_str!("../README.md")]

use std::any::TypeId;
use std::fmt::{Debug, Display, Formatter};
use std::mem::ManuallyDrop;
use std::ptr;

pub use conerror_macro::conerror;

/// Like [Error::chain], can be used in macro invocations while `#[conerror]` won't work.
#[macro_export]
macro_rules! chain {
    ($result:expr, $func_name:expr) => {
        $result.map_err(|e| conerror::Error::chain(e, file!(), line!(), $func_name, module_path!()))
    };
}

pub type Result<T> = std::result::Result<T, Error>;

/// Error with location information
pub struct Error(Box<Inner>);

impl Error {
    /// Create an [Error] with location information.
    pub fn new<T>(
        error: T,
        file: &'static str,
        line: u32,
        func: &'static str,
        module: &'static str,
    ) -> Self
    where
        T: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self(Box::new(Inner {
            source: error.into(),
            location: Some(vec![Location {
                file,
                line,
                func,
                module,
            }]),
        }))
    }

    /// Create an [Error] without location information
    pub fn plain<T>(error: T) -> Self
    where
        T: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self(Box::new(Inner {
            source: error.into(),
            location: None,
        }))
    }

    /// Same as [Error::new] if `error` is not of type [Error],
    /// otherwise add location information to `error` if not created by [Error::plain]
    pub fn chain<T>(
        error: T,
        file: &'static str,
        line: u32,
        func: &'static str,
        module: &'static str,
    ) -> Self
    where
        T: std::error::Error + Send + Sync + 'static,
    {
        if TypeId::of::<T>() == TypeId::of::<Self>() {
            let error = ManuallyDrop::new(error);
            // SAFETY: type checked
            let mut error = unsafe { ptr::read(&error as *const _ as *const Self) };
            if let Some(ref mut location) = error.0.location {
                location.push(Location {
                    file,
                    line,
                    func,
                    module,
                });
            }
            return error;
        }

        Self::new(error, file, line, func, module)
    }

    /// Return the location information
    pub fn location(&self) -> Option<&[Location]> {
        self.0.location.as_ref().map(|v| v.as_slice())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("source", &self.0.source)
            .field("location", &self.0.location)
            .finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.source, f)?;
        if let Some(ref location) = self.0.location {
            for (i, v) in location.iter().enumerate() {
                write!(f, "\n#{} {}", i, v)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.0.source)
    }
}

struct Inner {
    source: Box<dyn std::error::Error + Send + Sync>,
    location: Option<Vec<Location>>,
}

#[derive(Debug)]
pub struct Location {
    pub file: &'static str,
    pub line: u32,
    pub func: &'static str,
    /// module path for function, struct name for method
    pub module: &'static str,
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} {}::{}()",
            self.file, self.line, self.module, self.func
        )
    }
}
