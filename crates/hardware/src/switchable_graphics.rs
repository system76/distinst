use misc;
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

const POWER: &str = "etc/modprobe.d/system76-power.conf";
const PRIME_DISCRETE: &str = "etc/prime-discrete";

static MODPROBE_HYBRID: &[u8] = br#"# Automatically generated by distinst
options nvidia NVreg_DynamicPowerManagement=0x02
blacklist i2c_nvidia_gpu
alias i2c_nvidia_gpu off
"#;

static MODPROBE_INTEGRATED: &[u8] = br#"# Automatically generated by distinst
blacklist i2c_nvidia_gpu
blacklist nouveau
blacklist nvidia
blacklist nvidia-drm
blacklist nvidia-modeset
alias i2c_nvidia_gpu off
alias nouveau off
alias nvidia off
alias nvidia-drm off
alias nvidia-modeset off
"#;

/// Configure graphics mode if switchable graphics is supported.
pub fn configure_graphics(mount_dir: &Path) -> io::Result<bool> {
    let product_version = &*product_version();
    let switchable = has_switchable_graphics(product_version);

    if !switchable {
        return Ok(false);
    }

    let _ = fs::create_dir_all(mount_dir.join("etc/modprobe.d/"));

    if DEFAULT_INTEGRATED.contains(&product_version) {
        info!("disabling external NVIDIA graphics by default");
        fs::write(mount_dir.join(POWER), MODPROBE_INTEGRATED);
    } else {
        info!("settings module options for hybrid graphics mode");
        fs::write(mount_dir.join(POWER), MODPROBE_HYBRID)?;

        info!("configuring gpu-manager for hybrid graphics mode");
        fs::write(mount_dir.join(PRIME_DISCRETE), "on-demand")?;
    }

    Ok(true)
}

/// Products which support switchable graphics.
static SWITCHABLE_GRAPHICS: &[&str] = &[
    "addw1",
    "addw2",
    "gaze14",
    "gaze15",
    "oryp4",
    "oryp4-b",
    "oryp5",
    "oryp6",
];

/// Products which should default to integrated mode instead of hybrid mode.
static DEFAULT_INTEGRATED: &[&str] = &[
    "oryp4",
    "oryp4-b",
];

fn has_switchable_graphics(product: &str) -> bool { SWITCHABLE_GRAPHICS.contains(&product) }

/// Path where the product version can be obtained from the DMI.
const DMI_PATH_PRODUCT_VERSION: &str = "/sys/class/dmi/id/product_version";

fn product_version() -> String {
    let mut output = String::new();
    if let Ok(mut file) = misc::open(DMI_PATH_PRODUCT_VERSION) {
        let _ = file.read_to_string(&mut output);
        output = output.trim().into();
    }
    output
}
