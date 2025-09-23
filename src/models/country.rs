macro_rules! define_countries {
    ($(($variant:ident, $display_name:literal, $slug:literal, $country_code:literal)),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Country {
            $($variant,)*
        }

        impl Country {
            pub const COUNT: usize = count_items!($($variant)*);

            #[must_use]
            pub fn parse(string: &str) -> Option<Self> {
                match string {
                    $($display_name | $country_code => Some(Country::$variant),)*
                    _ => None,
                }
            }

            #[must_use]
            pub const fn display_name(&self) -> &'static str {
                match self {
                    $(Country::$variant => $display_name,)*
                }
            }

            #[must_use]
            pub const fn slug(&self) -> &'static str {
                match self {
                    $(Country::$variant => $slug,)*
                }
            }

            #[must_use]
            pub const fn code(&self) -> &'static str {
                match self {
                    $(Country::$variant => $country_code,)*
                }
            }

            #[must_use]
            pub const fn all() -> &'static [Self; Self::COUNT] {
                &[$(Country::$variant,)*]
            }

            #[must_use]
            pub const fn from_index(index: usize) -> Option<Self> {
                if index < Self::COUNT {
                    Some(Self::all()[index])
                } else {
                    None
                }
            }

        }

        impl std::fmt::Display for Country {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{display_name}", display_name = self.display_name())
            }
        }

        impl std::str::FromStr for Country {
            type Err = ();

            fn from_str(string: &str) -> Result<Self, Self::Err> {
                Self::parse(string).ok_or(())
            }
        }
    };
}

macro_rules! count_items {
    () => { 0 };
    ($head:tt $($tail:tt)*) => { 1 + count_items!($($tail)*) };
}

define_countries! {
    (Austria, "Austria", "austria", "at"),
    (Belgium, "Belgium", "belgium", "be"),
    (Bulgaria, "Bulgaria", "bulgaria", "bg"),
    (Croatia, "Croatia", "croatia", "hr"),
    (Cyprus, "Cyprus", "cyprus", "cy"),
    (CzechRepublic, "Czech Republic", "czech_republic", "cz"),
    (Denmark, "Denmark", "denmark", "dk"),
    (Estonia, "Estonia", "estonia", "ee"),
    (Finland, "Finland", "finland", "fi"),
    (France, "France", "france", "fr"),
    (Germany, "Germany", "germany", "de"),
    (Greece, "Greece", "greece", "gr"),
    (Hungary, "Hungary", "hungary", "hu"),
    (Ireland, "Ireland", "ireland", "ie"),
    (Italy, "Italy", "italy", "it"),
    (Latvia, "Latvia", "latvia", "lv"),
    (Lithuania, "Lithuania", "lithuania", "lt"),
    (Luxembourg, "Luxembourg", "luxembourg", "lu"),
    (Malta, "Malta", "malta", "mt"),
    (Netherlands, "Netherlands", "netherlands", "nl"),
    (Poland, "Poland", "poland", "pl"),
    (Portugal, "Portugal", "portugal", "pt"),
    (Romania, "Romania", "romania", "ro"),
    (Slovakia, "Slovakia", "slovakia", "sk"),
    (Slovenia, "Slovenia", "slovenia", "si"),
    (Spain, "Spain", "spain", "es"),
    (Sweden, "Sweden", "sweden", "se"),
    (Switzerland, "Switzerland", "switzerland", "ch"),
    (UnitedKingdom, "United Kingdom", "united_kingdom", "gb"),
    (Ukraine, "Ukraine", "ukraine", "ua")
}