#![doc = include_str!("../README.md")]

use std::any::TypeId;
use std::fmt::{Debug, Display, Formatter};
use std::mem::ManuallyDrop;
use std::ptr;

pub use conerror_macro::conerror;

pub type Result<T> = std::result::Result<T, Error>;

/// 带位置信息的错误
pub struct Error(Box<Inner>);

impl Error {
    /// 返回一个以 `error` 为底层错误，`file`, `line`, `func`, `module` 为位置信息的 [Error]
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

    /// 返回一个不需要位置信息的 [`Error`]
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
    /// 如果 `error` 的类型是 [`Error`]，并且不是创建自 [`Error::plain`]，则把 `file` `line` `func` `module` 追加到 `error` 的位置信息
    ///
    /// 如果 `error` 的类型不是 [`Error`]，则创建一个以 `error` 为底层错误，`file` `line` `func` 为初始位置的 [`Error`]
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
            // SAFETY: 已经检查了 error 的类型是 Error, 所以 read 是安全的
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

        Self(Box::new(Inner {
            source: Box::new(error),
            location: Some(vec![Location {
                file,
                line,
                func,
                module,
            }]),
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
        match self.0.location {
            Some(ref location) if !location.is_empty() => {
                writeln!(f)?;
                let idx = location.len() - 1;
                for v in &location[..idx] {
                    writeln!(f, "{}", v)?;
                }
                Display::fmt(&location[idx], f)?;
            }
            _ => (),
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
    /// 函数所属的模块或者 Struct
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

mod sealed {
    pub trait Sealed {}
}

pub trait ConerrorResult: sealed::Sealed {
    const ASSERT: () = ();
}

impl<T> sealed::Sealed for Result<T> {}

impl<T> ConerrorResult for Result<T> {}

pub struct SubstitutedImplTrait;
