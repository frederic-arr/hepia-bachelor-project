use std::ffi::CString;
use std::path::Path;

use loopdev::LoopControl;
use rustix::mount::MountFlags;

pub fn switchroot<Target, Init>(target: Target, init: Init)
where
    Target: AsRef<Path>,
    Init: AsRef<Path>,
{
    todo!()
}

pub enum SpecialFs {
    Sys,
    Proc,
    Dev,
    Cgroup2,
    Tmp,
    DevPts,
    Hugetlbfs,
    Bpf,
    Trace,
    Config,
    Debug,
    // SeLinux,
    Security,
}

impl SpecialFs {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Sys => "sysfs",
            Self::Proc => "proc",
            Self::Dev => "devtmpfs",
            Self::Cgroup2 => "cgroup2",
            Self::Tmp => "tmpfs",
            Self::DevPts => "devpts",
            Self::Hugetlbfs => "hugetlbfs",
            Self::Bpf => "bpf",
            Self::Trace => "tracefs",
            Self::Config => "configfs",
            Self::Debug => "debufs",
            Self::Security => "securityfs",
        }
    }
}

pub fn mount_special<Target>(
    fs_type: &SpecialFs,
    target: Target,
    flags: MountFlags,
    options: &[&str],
) -> std::io::Result<()>
where
    Target: AsRef<Path>,
{
    tracing::trace!("checking {}", target.as_ref().display());
    if !std::fs::exists(&target)? {
        tracing::trace!("creating {} directory", target.as_ref().display());
        std::fs::create_dir_all(&target)?;
    }

    let opts = CString::new(options.join(","))?;

    tracing::trace!(
        "mounting {} as {}",
        target.as_ref().display(),
        fs_type.as_str()
    );
    rustix::mount::mount(
        fs_type.as_str(),
        target.as_ref(),
        fs_type.as_str(),
        flags,
        (!options.is_empty()).then_some(opts.as_c_str()),
    )
    .map_err(Into::into)
}

pub fn mount_squashfs<Target, Image>(
    target: Target,
    image: Image,
    flags: MountFlags,
    options: &[&str],
) -> std::io::Result<()>
where
    Target: AsRef<Path>,
    Image: AsRef<Path>,
{
    if !std::fs::exists(&target)? {
        std::fs::create_dir_all(&target)?;
    }

    let lc = LoopControl::open()?;
    let ld = lc.next_free()?;
    ld.attach_file(image)?;

    let opts = CString::new(options.join(","))?;
    rustix::mount::mount(
        ld.path().unwrap(),
        target.as_ref(),
        "squashfs",
        flags,
        (!options.is_empty()).then_some(opts.as_c_str()),
    )
    .map_err(Into::into)
}
