use std::ffi::CString;
use std::path::Path;

use loopdev::{LoopControl, LoopDevice};
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
    Overlay,
    Iso,
    Squashfs,
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
            Self::Overlay => "overlay",
            Self::Iso => "iso9660",
            Self::Squashfs => "squashfs",
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

pub fn attach_loop<Image>(image: Image) -> std::io::Result<LoopDevice>
where
    Image: AsRef<Path>,
{
    let lc = LoopControl::open()?;
    let ld = lc.next_free()?;
    ld.with().read_only(true).attach(image)?;
    Ok(ld)
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
    if !std::fs::exists(&target).unwrap() {
        std::fs::create_dir_all(&target).unwrap();
    }

    let opts = CString::new(options.join(","))?;
    rustix::mount::mount(
        image.as_ref(),
        target.as_ref(),
        SpecialFs::Squashfs.as_str(),
        flags.union(MountFlags::RDONLY),
        (!options.is_empty()).then_some(opts.as_c_str()),
    )
    .map_err(Into::into)
}

pub fn mount_iso<Target, Image>(
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

    let opts = CString::new(options.join(","))?;
    rustix::mount::mount(
        image.as_ref(),
        target.as_ref(),
        SpecialFs::Iso.as_str(),
        flags.union(MountFlags::RDONLY),
        (!options.is_empty()).then_some(opts.as_c_str()),
    )
    .map_err(Into::into)
}

pub fn mount_overlayfs<Lower, Upper, Work, Target>(
    lower: &[Lower],
    writable: Option<(Upper, Work)>,
    target: Target,
    flags: MountFlags,
    options: &[&str],
) -> std::io::Result<()>
where
    Lower: AsRef<Path>,
    Upper: AsRef<Path>,
    Work: AsRef<Path>,
    Target: AsRef<Path>,
{
    if !std::fs::exists(&target)? {
        std::fs::create_dir_all(&target)?;
    }

    let lower = lower
        .iter()
        .map(|l| l.as_ref().display().to_string())
        .collect::<Vec<_>>()
        .join(":");

    let mut options = options.to_vec();

    let lower = format!("lowerdir={lower}");
    options.push(&lower);

    #[expect(
        clippy::branches_sharing_code,
        reason = "due to lifetimes, we cannot do it another way"
    )]
    let options = if let Some((upper, work)) = writable {
        if !std::fs::exists(&upper)? {
            std::fs::create_dir_all(&upper)?;
        }

        if !std::fs::exists(&work)? {
            std::fs::create_dir_all(&work)?;
        }

        let a = format!(
            "upperdir={},workdir={}",
            upper.as_ref().display(),
            work.as_ref().display(),
        );
        options.push(&a);
        CString::new(options.join(","))?
    } else {
        CString::new(options.join(","))?
    };

    rustix::mount::mount(
        SpecialFs::Overlay.as_str(),
        target.as_ref(),
        SpecialFs::Overlay.as_str(),
        flags,
        Some(options.as_c_str()),
    )
    .map_err(Into::into)
}
