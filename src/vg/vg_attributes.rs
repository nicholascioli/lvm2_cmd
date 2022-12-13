use crate::{vg::VolumeGroupAccessMode, AllocationPolicy, Permissions, TryFromChar};

#[derive(Clone, Debug)]
pub struct VolumeGroupAttributes {
    pub permissions: Permissions,

    /// Iff this volume group be resized.
    pub is_resizeable: bool,

    /// Iff this volume group
    pub is_exported: bool,

    /// Iff one or more physical volumes belonging to the volume group are missing from the system.
    pub is_partial: bool,
    pub allocation_policy: AllocationPolicy,
    pub access_mode: VolumeGroupAccessMode,
}

pub(crate) fn deserialize_vg_attrs<'de, D>(
    deserializer: D,
) -> Result<VolumeGroupAttributes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // define a visitor that deserializes
    // `ActualData` encoded as json within a string
    struct AttrStringVisitor;

    impl<'de> serde::de::Visitor<'de> for AttrStringVisitor {
        type Value = VolumeGroupAttributes;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing vg attribute data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut chars = v.chars();

            let permissions = Permissions::try_from_char(
                chars
                    .next()
                    .ok_or(E::custom("could not get group permissions attribute"))?,
            )
            .map_err(E::custom)?;

            let is_resizeable = match chars
                .next()
                .ok_or(E::custom("could not get group resize attribute"))?
            {
                '-' => false,
                'z' => true,

                _ => return Err(E::custom("invalid flag for IsResizeable")),
            };

            let is_exported = match chars
                .next()
                .ok_or(E::custom("could not get group export attribute"))?
            {
                '-' => false,
                'x' => true,

                _ => return Err(E::custom("invalid flag for IsExported")),
            };

            let is_partial = match chars
                .next()
                .ok_or(E::custom("could not get group partial attribute"))?
            {
                '-' => false,
                'p' => true,

                _ => return Err(E::custom("invalid flag for IsPartial")),
            };

            let allocation_policy = AllocationPolicy::try_from_char(
                chars
                    .next()
                    .ok_or(E::custom("could not get allocation policy attribute"))?,
            )
            .map_err(E::custom)?;

            let access_mode = VolumeGroupAccessMode::try_from_char(
                chars
                    .next()
                    .ok_or(E::custom("could not get group access mode attribute"))?,
            )
            .map_err(E::custom)?;

            Ok(VolumeGroupAttributes {
                permissions,
                is_resizeable,
                is_exported,
                is_partial,
                allocation_policy,
                access_mode,
            })
        }
    }

    // use our visitor to deserialize an `ActualValue`
    deserializer.deserialize_any(AttrStringVisitor)
}
