mod country;
mod product;
mod category;
mod catalog;

#[cfg(runtime)]
type String = &'static str;
#[cfg(not(runtime))]
type String = std::string::String;

#[cfg(runtime)]
type Categories = &'static[usize];
#[cfg(not(runtime))]
type Categories = Vec<String>;

#[cfg(runtime)]
type Array<T> = &'static[T];
#[cfg(not(runtime))]
type Array<T> = Vec<T>;

pub use self::country::Country;
pub use self::product::Product;
pub use self::category::Category;
pub use self::catalog::Catalog;