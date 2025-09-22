/// ## General
///
/// Core interface components shared across all platforms,  
/// such as headers, bumpers, and content containers.
///
pub mod general;

/// ## Mobile
///
/// Components tailored for mobile platforms,  
/// for example the [`MobileNavigator`].
///
pub mod mobile;

/// ## Desktop
///
/// Components designed specifically for desktop platforms,  
/// such as the [`DesktopNavigator`].
///
pub mod desktop;

/// ## Web
///
/// Components optimized for web environments,  
/// for instance the [`WebNavigator`].
///
pub mod web;

/// ## System Integration
///
/// OS-level integration components that handle  
/// system input and replacement behaviors.
///
pub mod system;
