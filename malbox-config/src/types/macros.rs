#[macro_export]
macro_rules! impl_display_fromstr {
    ($type:ty, $( $variant:ident => $str:expr ),+ ) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $( Self::$variant => write!(f, $str), )+
                }
            }
        }

        impl std::str::FromStr for $type {
            type Err = $crate::ConfigError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    $( $str => Ok(Self::$variant), )+
                    _ => Err($crate::ConfigError::InvalidValue {
                        field: stringify!($type).to_string(),
                        message: format!("Invalid value: {}", s),
                    }),
                }
            }
        }
    };
}
