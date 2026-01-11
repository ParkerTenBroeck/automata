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

#[macro_export]
macro_rules! dual_enum_serde {
        (
        $( {$(#[$serde_specific:meta])*} )?
        $(#[$enum_meta:meta])*
        $vis:vis enum $Name:ident $(<$($gen:tt),*>)?
        {
            $(
                $(#[$variant_meta:meta])*
                $Variant:ident
                $(
                    // Tuple variant: Variant(T1, T2, ...)
                    ( $(
                        $(#[$tfield_meta:meta])*
                        $tfield_ty:ty
                    ),* $(,)? )
                )?
                $(
                    // Struct variant: Variant { a: T, b: U, ... }
                    { $(
                        $(#[$sfield_meta:meta])*
                        $sfield_vis:vis $sfield_name:ident : $sfield_ty:ty
                    ),* $(,)? }
                )?
            ),* $(,)?
        }
    ) => {
        #[cfg(feature = "serde")]
        $(#[$enum_meta])*
        #[derive(serde::Serialize, serde::Deserialize)]
        $( $(#[$serde_specific])* )?
        $vis enum $Name $(<$($gen),*>)? {
            $(
                $(#[$variant_meta])*
                $Variant
                $(
                    (
                        $(
                            $(#[$tfield_meta])*
                            $tfield_ty
                        ),*
                    )
                )?
                $(
                    {
                        $(
                            $(#[$sfield_meta])*
                            $sfield_vis $sfield_name: $sfield_ty
                        ),*
                    }
                )?
            ),*
        }

        #[cfg(not(feature = "serde"))]
        $(#[$enum_meta])*
        $vis enum $Name $(<$($gen),*>)? {
            $(
                // strip variant + field attrs in non-serde version
                $Variant
                $(
                    (
                        $(
                            $tfield_ty
                        ),*
                    )
                )?
                $(
                    {
                        $(
                            $sfield_vis $sfield_name: $sfield_ty
                        ),*
                    }
                )?
            ),*
        }
    };
}
