/// Convert a set of types that implement [`Module`]
/// to a set of [`ResolveModule`] instances.
///
/// # Example
///
/// ```rust
/// use rudi::{modules, Module};
///
/// struct MyModule;
///
/// impl Module for MyModule {
///     fn providers() -> Vec<rudi::DynProvider> {
///         Vec::new()
///     }
/// }
///
/// # fn main() {
/// let _: Vec<rudi::ResolveModule> = modules![MyModule];
/// # }
/// ```
///
/// [`Module`]: crate::Module
/// [`ResolveModule`]: crate::ResolveModule
#[macro_export]
macro_rules! modules {
    () => {
        vec![]
    };
    ($($module:ty),+ $(,)?) => {
        vec![$(
            $crate::ResolveModule::new::<$module>()
        ),+]
    };
}

/// Convert a set of instances that implement `Into<DynProvider>`
/// to a set of [`DynProvider`] instances
///
/// # Example
///
/// ```rust
/// use rudi::{providers, singleton};
///
/// # fn main() {
/// let _: Vec<rudi::DynProvider> = providers![singleton(|_| "Hello")];
/// # }
/// ```
///
/// [`DynProvider`]: crate::DynProvider
#[macro_export]
macro_rules! providers {
    () => {
        vec![]
    };
    ($($provider:expr),+ $(,)?) => {
        vec![$(
            <$crate::DynProvider as ::core::convert::From<_>>::from($provider)
        ),+]
    };
}

/// Convert a set of types that implement [`DefaultProvider`]
/// to a set of [`DynProvider`] instances
///
/// # Example
///
/// ```rust
/// use rudi::{components, Transient};
///
/// #[Transient]
/// struct A;
///
/// # fn main() {
/// let _: Vec<rudi::DynProvider> = components![A];
/// # }
/// ```
///
/// [`DefaultProvider`]: crate::DefaultProvider
/// [`DynProvider`]: crate::DynProvider
#[macro_export]
macro_rules! components {
    () => {
        vec![]
    };
    ($($component:ty),+ $(,)?) => {
        vec![$(
            <$crate::DynProvider as ::core::convert::From<_>>::from(
                <$component as $crate::DefaultProvider>::provider()
            )
        ),+]
    };
}
