pub trait EnumFromStr: Sized {
    fn enum_str(str: &str) -> Result<Self, String>;
}

#[macro_export]
macro_rules! value_enum {
    ($name:ident, $( $(#[$meta:meta])* $value:ident ),* $(,)?) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum $name {
            $(
                $(#[$meta])*
                $value,
            )*
        }

        impl EnumFromStr for $name {
            fn enum_str(str: &str) -> Result<Self, String> {
                match str {
                    $(
                        $(#[$meta])*
                        stringify!($value) => Ok($name::$value),
                    )*
                    _ => Err(format!("Unknown value {}", str))
                }
            }
        }

        impl ToString for $name {
            fn to_string(&self) -> String {
                match self {
                    $(
                        $(#[$meta])*
                        $name::$value => stringify!($value).to_string(),
                    )*
                }
            }
        }
    };
}

pub(crate) use value_enum;
