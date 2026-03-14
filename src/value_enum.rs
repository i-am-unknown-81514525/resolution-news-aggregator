use std::fmt::{write, Display};

pub trait EnumFromStr : Sized {
    fn enum_str(str: &str) -> Result<Self, String>;
}

#[macro_export]
macro_rules! value_enum {
    ($name:ident, $( $value:ident ),*) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum $name {
            $(
                $value,
            )*
        }

        impl EnumFromStr for $name {
            fn enum_str(str: &str) -> Result<Self, String> {
                match str {
                    $(
                        stringify!($value) => Ok($name::$value),
                    )*
                    _ => Err(format!("Unknown value {}", str))
                }
            }
        }

        // impl ::std::fmt::Display for $name {
        //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         match self {
        //             $(
        //                 $name::$value => write!(f, "{}.{}", stringify!($name), self.to_string()),
        //             )*
        //         }
        //     }
        // }

        impl ToString for $name {
            fn to_string(&self) -> String {
                match self {
                    $(
                        $name::$value => stringify!($value).to_string(),
                    )*
                }
            }
        }
    };
}

pub(crate) use value_enum; 