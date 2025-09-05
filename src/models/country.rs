macro_rules! define_countries {
    ($($variant:ident),* $(,)?) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Country {
            $($variant),*
        }


        impl Country {
            pub const COUNT: usize = [$(stringify!($variant)),*].len();

            pub fn parse(text: &str) -> Option<Self> {
                let normalized = heck::AsSnakeCase(text).to_string();
                match normalized.as_str() {
                    $(
                        stringify!(const_str::convert_case!(snake, $variant)) => Some(Self::$variant),
                    )*
                    _ => None,
                }
            }

            pub const fn index(self) -> usize {
                self as usize
            }

            pub const fn from_index(index: usize) -> Option<Self> {
                if index < Self::COUNT {
                    Some(unsafe { std::mem::transmute(index as u8) })
                } else {
                    None
                }
            }

            pub fn all() -> impl Iterator<Item = Self> {
                (0..Self::COUNT).filter_map(Self::from_index)
            }

            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!(const_str::convert_case!(title, $variant))),*
                }
            }

            pub const fn icon(self) -> &'static str {
                match self {
                    $(Self::$variant => concat!(stringify!(const_str::convert_case!(snake, $variant), "_flag"))),*
                }
            }

            pub fn from_str(text: &str) -> Option<Self> {
                Self::parse(text)
            }
        }
    };
}

define_countries! {
    Austria,
    Belgium,
    Bulgaria,
    Croatia,
    Cyprus,
    Czech,
    Denmark,
    Estonia,
    Finland,
    France,
    Germany,
    Greece,
    Hungary,
    Ireland,
    Italy,
    Latvia,
    Lithuania,
    Luxembourg,
    Malta,
    Netherlands,
    Poland,
    Portugal,
    Romania,
    Slovakia,
    Slovenia,
    Spain,
    Sweden,
    Switzerland,
    UnitedKingdom
}

impl std::fmt::Display for Country {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.name())
    }
}

impl std::str::FromStr for Country {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Self::parse(string).ok_or(())
    }
}