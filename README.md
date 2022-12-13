# lvm2_cmd

This crate facilitates working with the suite of LVM2 commands programmatically
in safe Rust.

Interacting with `LogicalVolumes` can be done either through the owning `VolumeGroup`,
or through the use of static functions of `LogicalVolume`.

## Limitations

- LVM2 all but requires root access in order to fully manipulate both the
creation / deletion of resources (through the use of the kernel `devicemapper`)
and for the querying of resources (through the `noatime` reading of block
devices.). This crate attempts to sanitize input, but be warned that it has
not been thoroughly tested.
- LVM2 enforces that all resource names adhere to the following regex
`^[0-9a-zA-Z_.+\-]$`. This is enforced for any function that takes a resource name.
- LVM2 enforces that a logical volume's capacity be a multiple of 512. This is enforced
for any function that takes in a capacity for a logical volume.

## Examples

Some examples of how to use this library are shown below:

```rust
use lvm2_cmd::{vg::VolumeGroup, lv::{LogicalVolume, LVCreateOptions}};

// List resources
let vgs = VolumeGroup::list()?;
let lvs = LogicalVolumes::list()?;

// List resources for a specific volume group
let vg_test = VolumeGroup::get("test")?;
let lvs_of_test = vg_test.list_lvs()?; // or LogicalVolume::list_for_vg("test")?

// Create a logical volume
let vg = VolumeGroup::get("test");
let lv = vg.add_lv(                    // or LogicalVolume::create("test",
    LVCreateOptions {
        activate: true,
        capacity_bytes: 536870912,     // 512 MB
        name: "lv01".into(),
        tags: vec![],
    }
)?;

```
