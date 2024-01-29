#![doc = include_str!("../README.md")]

use std::any::TypeId;
use std::fmt::{Debug, Display, Formatter};
use std::mem::ManuallyDrop;
use std::ptr;

pub type Result<T> = std::result::Result<T, Error>;

pub use conerror_macro::conerror;

/// 带位置信息的错误
pub struct Error(Box<Inner>);

impl Error {
    /// 创建一个不需要位置信息的 [`Error`]
    pub fn plain<T>(error: T) -> Self
    where
        T: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self(Box::new(Inner {
            source: error.into(),
            location: None,
        }))
    }

    /// 给 `error` 加上位置信息
    ///
    /// 如果 `error` 的类型是 [`Error`]，并且不是创建自 [`Error::plain`]，则把 `file` `line` `func` 追加到 `error` 的位置信息
    ///
    /// 如果 `error` 的类型不是 [`Error`]，则创建一个以 `error` 为底层错误，`file` `line` `func` 为初始位置的 [`Error`]
    pub fn chain<T>(error: T, file: &'static str, line: u32, func: &'static str) -> Self
    where
        T: std::error::Error + Send + Sync + 'static,
    {
        if TypeId::of::<T>() == TypeId::of::<Self>() {
            let error = ManuallyDrop::new(error);
            // SAFETY: 已经检查了 error 的类型是 Error, 所以 read 是安全的
            let mut error = unsafe { ptr::read(&error as *const _ as *const Self) };
            if let Some(ref mut location) = error.0.location {
                location.push(Location { file, line, func });
            }
            return error;
        }

        Self(Box::new(Inner {
            source: Box::new(error),
            location: Some(vec![Location { file, line, func }]),
        }))
    }

    /// 返回这个错误的位置信息
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
            writeln!(f)?;
            for v in location {
                writeln!(f, "{}", v)?;
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
    /// 底层的错误
    source: Box<dyn std::error::Error + Send + Sync>,
    /// 位置信息
    location: Option<Vec<Location>>,
}

/// 错误位置
#[derive(Debug)]
pub struct Location {
    /// 错误所在的文件
    pub file: &'static str,
    /// 错误所在的行
    pub line: u32,
    /// 错误所在的函数
    pub func: &'static str,
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} {}()", self.file, self.line, self.func)
    }
}
