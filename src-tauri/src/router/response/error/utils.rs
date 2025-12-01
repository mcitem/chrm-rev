use axum::http::StatusCode;
use std::fmt::Debug;

pub trait AppError: Debug {
    fn code(&self) -> StatusCode;

    fn msg(&self) -> Option<String> {
        None
    }

    fn error(&self) -> String {
        format!("{:?}", self)
    }
}

#[macro_export(local_inner_macros)]
macro_rules! impl_into_app_error {
    (
        for $target:ty;
        $(
            $middle:ty => [
                $(
                    $source:ty,
                )+
            ],
        )+
    ) => {
        $(
            $(
                impl From<$source> for $target {
                    fn from(value: $source) -> Self {
                        Into::<$middle>::into(value).into()
                    }
                }
            )+
        )+
    };
}
pub trait IntoOpt {
    fn into_opt(self) -> Option<String>;
}

impl IntoOpt for &'static str {
    fn into_opt(self) -> Option<String> {
        Some(self.to_owned())
    }
}

impl IntoOpt for String {
    fn into_opt(self) -> Option<String> {
        Some(self)
    }
}

impl IntoOpt for Option<String> {
    fn into_opt(self) -> Option<String> {
        self
    }
}

#[macro_export(local_inner_macros)]
macro_rules! impl_app_error {
    (
        for $target:ty;
        $(
            $code:ident => [
                $(
                    $variant:ident $( ( $($variant_pat:pat),+ $(,)? ) )? ; $msg:expr,
                )+
            ],
        )*
        $(
            @delegates;
            $(
                $delegate_variant:ident,
            )+
        )?
    ) => {
        impl crate::router::response::error::utils::AppError for $target {
            #[allow(unused_variables)]
            fn code(&self) -> StatusCode {
                match self {
                    $(
                        $(
                            Self::$variant $( ( $($variant_pat),+ ) )? => axum::http::StatusCode::$code,
                        )+
                    )*
                    $(
                        $(
                            Self::$delegate_variant(__delegate, ..) => __delegate.code(),
                        )+
                    )?
                }
            }

            fn msg(&self) -> Option<String> {
                match self {
                    $(
                        $(
                            Self::$variant $( ( $($variant_pat),+ ) )? => crate::router::response::error::utils::IntoOpt::into_opt($msg),
                        )+
                    )*
                    $(
                        $(
                            Self::$delegate_variant(__delegate, ..) => __delegate.msg(),
                        )+
                    )?
                }
            }
        }
    };
}
