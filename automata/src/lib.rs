pub mod automatan;
pub mod loader;


#[macro_export]
macro_rules! dual_struct_serde {
    ($({$(#[$serde_specific:meta])*})?
        $(#[$struct_meta:meta])*
        $vis:vis struct $Name:ident $(<$($gen:tt),*>)?
        {
            $(
                $(#[$field_meta:meta])*
                $fvis:vis $fname:ident : $fty:ty
            ),* $(,)?
        }
    ) => {
        #[cfg(feature = "serde")]
        $(#[$struct_meta])*
        $( $(#[$serde_specific])* )?
        #[derive(serde::Serialize, serde::Deserialize)]
        $vis struct $Name $(<$($gen)*>)? {
            $(
                $(#[$field_meta])*
                $fvis $fname: $fty
            ),*
        }

        #[cfg(not(feature = "serde"))]
        $(#[$struct_meta])*
        $vis struct $Name $(<$($gen)*>)? {
            $(
                $fvis $fname: $fty
            ),*
        }
    };
}