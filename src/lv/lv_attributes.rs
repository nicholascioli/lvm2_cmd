use crate::{
    lv::{LVState, LVStatus, LVVolumeType},
    AllocationPolicy, Permissions, TryFromChar,
};

#[derive(Clone, Debug)]
// TODO: Fill the rest out
pub struct LogicalVolumeAttributes {
    pub volume_type: LVVolumeType,
    pub permissions: Permissions,
    pub allocation_policy: AllocationPolicy,
    pub is_fixed_minor: bool,
    pub state: LVState,
    pub status: LVStatus,
}

/// Deserialize a [LogicalVolume]'s attributes from an attribute string
pub(crate) fn deserialize_lv_attrs<'de, D>(
    deserializer: D,
) -> Result<LogicalVolumeAttributes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // define a visitor that deserializes
    // `ActualData` encoded as json within a string
    struct AttrStringVisitor;

    impl<'de> serde::de::Visitor<'de> for AttrStringVisitor {
        type Value = LogicalVolumeAttributes;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string containing lv attribute data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            #[cfg(feature = "logging")]
            log::debug!("deserializing attr string: {}", v);

            let mut chars = v.chars();

            let volume_type = LVVolumeType::try_from_char(
                chars
                    .next()
                    .ok_or(E::custom("could not get volume type attribute"))?,
            )
            .map_err(E::custom)?;

            let permissions = Permissions::try_from_char(
                chars
                    .next()
                    .ok_or(E::custom("could not get volume permissions attribute"))?,
            )
            .map_err(E::custom)?;

            let allocation_policy = AllocationPolicy::try_from_char(
                chars
                    .next()
                    .ok_or(E::custom("could not get allocation policy attribute"))?,
            )
            .map_err(E::custom)?;

            let is_fixed_minor = match chars
                .next()
                .ok_or(E::custom("could not get fixed minor attribute"))?
            {
                '-' => false,
                'm' => true,
                _ => return Err(E::custom("invalid flag for IsFixedMinor")),
            };

            let state = LVState::try_from_char(
                chars
                    .next()
                    .ok_or(E::custom("could not get state attribute"))?,
            )
            .map_err(E::custom)?;

            let status = LVStatus::try_from_char(
                chars
                    .next()
                    .ok_or(E::custom("could not get status attribute"))?,
            )
            .map_err(E::custom)?;

            Ok(LogicalVolumeAttributes {
                volume_type,
                permissions,
                allocation_policy,
                is_fixed_minor,
                state,
                status,
            })
        }
    }

    // use our visitor to deserialize an `ActualValue`
    deserializer.deserialize_any(AttrStringVisitor)
}
